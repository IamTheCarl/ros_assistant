import rclpy
import rclpy.node
from rclpy.executors import ExternalShutdownException
from rcl_interfaces.msg import ParameterDescriptor, IntegerRange
from rclpy.parameter import Parameter
from rclpy.executors import MultiThreadedExecutor
from rclpy.callback_groups import ReentrantCallbackGroup
import asyncio
from websockets import server
from websockets.exceptions import ConnectionClosedOK
import cbor2
from cbor2 import CBORDecodeError
from .schema_gen import generate_schema
from .class_importing import import_message_class
import voluptuous as vol
import json
from .dictionaries_to_ros_messages import dictionary_to_ros_message
from .ros_messages_to_dictionaries import ros_message_to_dictionary
from homeassistant_interface.srv import JsonServiceCall

SERVICE_TO_FORWARD_TO_HA_CONFIG_SCHEMA = vol.Schema({
    vol.Required('ros_interface'): vol.Schema({
        vol.Required('package'): str,
        vol.Required('service'): str,
        vol.Required('server_path'): str,
    }),
    vol.Optional('description'): str,
    vol.Optional('example'): str,
    vol.Optional('timeout_seconds'): int,
    vol.Optional('fields'): {
        # FIXME this will only allow one field to be presented.
        str: vol.Schema({
            vol.Optional('description'): str,
            vol.Optional('example'): str,
        }),
    }
})

INCOMING_MESSAGE_SCHEMA = vol.Any(
    vol.Schema({
        vol.Required('type'): 'call_service',
        vol.Required('instance_id'): int,
        vol.Required('service_name'): str,
        vol.Required('request'): dict,
    }),
    vol.Schema({
        vol.Required('type'): 'respond_service',
        vol.Required('instance_id'): int,
        vol.Required('response'): vol.Any(
            dict,  # A proper response
            # Also a proper response but this service doesn't provide any
            # return value.
            None,
            str,  # Indicates an error
        ),
    })
)


class LocalServiceSubscription:
    def __init__(self, ros_subscription, request_class, timeout):
        self.ros_subscription = ros_subscription
        self.request_class = request_class
        self.timeout = timeout

    def call(self, request_dictionary, logger):
        request_instance = dictionary_to_ros_message(
            self.request_class, request_dictionary, logger)
        service = self.ros_subscription.call_async(request_instance)
        if self.timeout is not None:
            # We have a timeout. Wrap it up in this.
            service = asyncio.wait_for(service, timeout=self.timeout)

        return service


class HomeAssistantBridge(rclpy.node.Node):
    def __init__(self):
        super().__init__('home_assistant_bridge')

        self._local_service_subscriptions = {}
        self._configuration_advertisement = {
            "provided_services": {},
            "provided_topics": {},
            "expected_topics": {},
        }

        self._socket = None
        self._event_loop = asyncio.get_event_loop()

        self._next_outgoing_service_call_id = 0
        self._pending_outgoing_service_calls = {}
        self._home_assistant_service_provider = self.create_service(
            JsonServiceCall, 'call_homeassistant_service',
            self._sync_call_service_on_home_assistant,
            callback_group=ReentrantCallbackGroup())

        self._load_services_to_forward_to_ha()

        self.get_logger().info("Home Assistant Bridge started.")

    def _declare_parameter_list(self, name, description):
        self.declare_parameters('', [(
            name,
            Parameter.Type.STRING_ARRAY,
            ParameterDescriptor(description=description)
        )])

    def _load_services_to_forward_to_ha(self):
        self._declare_parameter_list(
            'services_to_forward_to_ha',
            'Services on our robot to be called from Home Assistant')

        services_to_forward_from_ha = self \
            .get_parameter('services_to_forward_to_ha') \
            .get_parameter_value().string_array_value
        for service in services_to_forward_from_ha:
            try:
                service_config = json.loads(service)
            except json.JSONDecodeError as e:
                self.get_logger().error(
                    ("Failed to decode configuration "
                     f"for service: {e}\n{service}"))
                continue

            try:
                SERVICE_TO_FORWARD_TO_HA_CONFIG_SCHEMA(service_config)
            except vol.error.Invalid as e:
                self.get_logger().error(
                    (f"Failed to validate configuration for service: {e}")
                )
                continue

            ros_interface = service_config['ros_interface']
            package = ros_interface['package']
            service_name = ros_interface['service']
            server_path = ros_interface['server_path']
            request_field_meta = service_config['fields']
            timeout_seconds = service_config.get('timeout_seconds')

            service_description = service_config.get(
                'description', f"Calls the `{service_name}` service")
            service_example = service_config.get('example', '{}')

            service_class = import_message_class(
                "srv", package, service_name, self.get_logger())

            # None indicates the import failed.
            # The `import_message_class` function will automatically log
            # failures, so there's no need to log it here.
            if service_class is not None:
                self.get_logger() \
                    .info(f"Forward {server_path}:{package}/{service_name} \
                        to Home Assistant")

                request = generate_schema(
                    service_class.Request, self.get_logger())

                # Get additional information about fields.
                for field_name, field_info in request.items():
                    meta = request_field_meta.get(field_name)
                    if meta is not None:
                        field_info['description'] = meta.get('description', '')
                        field_info['example'] = meta.get('example', '')

                response = generate_schema(
                    service_class.Response, self.get_logger())

                advertisement = {
                    'request': request,
                    'response': response,
                    'description': service_description,
                    'example': service_example
                }

                service_description = service_config.get('description')
                if service_description is not None:
                    advertisement['description'] = service_description

                service_example = service_config.get('example')
                if service_example is not None:
                    advertisement['example'] = service_example

                provided_services = \
                    self._configuration_advertisement["provided_services"]
                provided_services[server_path] = advertisement

                self._local_service_subscriptions[server_path] = \
                    LocalServiceSubscription(
                        self.create_client(service_class, server_path),
                        service_class.Request,
                        timeout_seconds)

    def _sync_call_service_on_home_assistant(self, request, response):
        future = asyncio.run_coroutine_threadsafe(
            self._call_service_on_home_assistant(request, response),
            self._event_loop)
        result = future.result()
        return result

    async def _call_service_on_home_assistant(self, request, response):
        try:
            if self._socket is not None:
                service_domain = request.domain
                service_name = request.name
                return_response = request.return_response
                request = request.json_request
                request = json.loads(request)

                # Get our request ready.
                instance_id = self._next_outgoing_service_call_id
                self._next_outgoing_service_call_id += 1

                request = {
                    'type': 'call_service',
                    'instance_id': instance_id,
                    'domain': service_domain,
                    'name': service_name,
                    'responds': return_response,
                    'request': request,
                }
                request_message = cbor2.dumps(request)

                # Be ready to receive the response before we send the request.
                # This will prevent race conditions where the response could
                # somehow come in before we're ready.
                future = asyncio.Future()
                self._pending_outgoing_service_calls[instance_id] = future

                self.get_logger().info(
                    f"Home Assistant Request: {request}")

                await self._socket.send(request_message)
                result = await future

                self.get_logger().info(
                    f"Home Assistant Response: {request} - {result}")

                match result:
                    case None:
                        # This indicates we got disconnected.
                        response.status = JsonServiceCall.Response. \
                            NOT_CONNECTED
                    case dict():
                        # We got a real response.
                        ha_response = result['response']

                        match ha_response:
                            case str():
                                response.status = JsonServiceCall.Response \
                                    .SERVICE_FAILURE
                                response.result = ha_response
                            case dict():
                                response_string = json.dumps(ha_response)

                                response.status = JsonServiceCall.Response \
                                    .SUCCESS
                                response.result = response_string
            else:
                response.status = JsonServiceCall.Response.NOT_CONNECTED
        except json.JSONDecodeError as e:
            response.status = JsonServiceCall.Response.BAD_JSON
            response.result = f"{e}"

        return response

    async def _call_local_service(self, service_call, socket):

        service_name = service_call['service_name']
        instance_id = service_call['instance_id']
        request = service_call['request']

        self.get_logger().info(
            f"Call service `{service_name}` with request {request}")

        service = self._local_service_subscriptions.get(service_name)
        if service is None:
            self.get_logger() \
                .error((
                    "Service call was for service "
                    f"we do not provide: {service_call}"))
            await socket.send(cbor2.dumps({
                'type': 'respond_service',
                'instance_id': instance_id,
                'response': 'not_provided',
            }))
            return

        # We will await this later.
        service = service.call(request, self.get_logger())

        async def call_service():
            try:
                response = await service
                response = ros_message_to_dictionary(response)
                response = {
                    'type': 'respond_service',
                    'instance_id': instance_id,
                    'response': response
                }
            except asyncio.TimeoutError:
                response = {
                    'type': 'respond_service',
                    'instance_id': instance_id,
                    'response': "Timed out"
                }

            self.get_logger().info(
                (f"Respond service `{service_name}` with request {request}: "
                 f"{response}"))

            message = cbor2.dumps(response)
            await socket.send(message)

        asyncio.create_task(call_service())

    async def run_websocket_server(self):
        self.declare_parameter(
            'port',
            8080,
            descriptor=ParameterDescriptor(
                description=("Port number for the Home Assistant "
                             "integration to connect to"),
                integer_range=[IntegerRange(
                    from_value=0, to_value=2**16, step=1)]
            ),
        )
        port = self.get_parameter('port').get_parameter_value().integer_value

        self.declare_parameter(
            'interface',
            '0.0.0.0',
            descriptor=ParameterDescriptor(
                description=("Network interface that Home Assistant should "
                             "connect through"),
            ),
        )
        interface = self.get_parameter(
            'port').get_parameter_value().string_value

        async with server.serve(self._handle_connection, interface, port):
            try:
                # Wait forever.
                await asyncio.get_running_loop().create_future()
            except KeyboardInterrupt:
                # Triggered by SIGINT.
                pass

    async def process_service_call_response(self, message):
        instance_id = message['instance_id']
        pending_future = self._pending_outgoing_service_calls.pop(
            instance_id, None)

        if pending_future:
            # The service may be an error string. Error handling will be done
            # in the service method, not here.
            pending_future.set_result(message)
        else:
            self.get_logger().warn(
                ("Received message response for "
                 f"non-pending message: {message}"))

    async def _handle_connection(self, socket):
        # We only permit one connection at a time.
        # It would be confusing if we wanted to call a service on
        # Home Assistant and didn't know which one to call it on.
        if self._socket is None:
            self._socket = socket
            try:
                self.get_logger().info(
                    "Client connected: {}".format(socket.remote_address))

                # Advertise our configuration.
                await socket.send(cbor2
                                  .dumps(self._configuration_advertisement))

                while True:
                    # Get the message from Home Assistant.
                    message = await socket.recv()

                    try:
                        message = cbor2.loads(message)
                        INCOMING_MESSAGE_SCHEMA(message)

                        # Any other case should have been rejected by the
                        # message schema.
                        match message['type']:
                            case 'call_service':
                                await self._call_local_service(message, socket)
                            case 'respond_service':
                                await self \
                                    .process_service_call_response(message)
                    except CBORDecodeError as e:
                        self.get_logger().error(
                            f"Failed to decode incoming message: {e}")
                    except (vol.error.Invalid, ValueError) as e:
                        self.get_logger().error(
                            f"Failed to validate incoming message: {e}")
            except ConnectionClosedOK:
                # This is not an error.
                # This is how a connection is closed.
                self.get_logger().info("Connection closed.")
            finally:
                # Abort all pending service requests.
                for _instance_id, pending_future in self.\
                        _pending_outgoing_service_calls.items():
                    pending_future.set_result(None)
                self._pending_outgoing_service_calls.clear()

                # We will now allow another connection.
                self._socket = None


def spin_ros(node):
    executor = MultiThreadedExecutor()
    executor.add_node(node)

    try:
        executor.spin()
    except (KeyboardInterrupt, ExternalShutdownException):
        # Triggered by SIGINT and SIGKILL, these are situations in which we
        # should gracefully shutdown.
        pass


async def async_main(args):
    rclpy.init(args=args)
    node = HomeAssistantBridge()

    websocket_service = asyncio.create_task(node.run_websocket_server())
    ros_spinner = asyncio.create_task(asyncio.to_thread(spin_ros, node))

    # Exits whenever the first one finishes
    await asyncio.wait(
        [ros_spinner, websocket_service],
        return_when=asyncio.FIRST_COMPLETED
    )


def main(args=None):
    asyncio.run(async_main(args))


if __name__ == '__main__':
    main()
