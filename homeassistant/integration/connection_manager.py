from .feild_metadata import collect_metadata_for_fields
from .schema_decode import decode_schema, SCHEMA_VALIDATOR
from .const import DOMAIN
import voluptuous as vol
from cbor2 import CBORDecodeError
import cbor2
from .const import DEFAULT_TIMEOUT_SECONDS
import asyncio
from websockets.exceptions import InvalidURI, InvalidHandshake, \
    ConnectionClosedError, ConnectionClosedOK
import websockets
import logging
from homeassistant.helpers.service import async_set_service_schema
from homeassistant.core import HomeAssistant, SupportsResponse, \
    HomeAssistantError, ServiceCall

_LOGGER = logging.getLogger(__name__)

PROVIDED_SERVICE_VALIDATOR = vol.Schema({}, extra=vol.ALLOW_EXTRA)
PROVIDED_SERVICE_VALIDATOR.extend({
    str: vol.Schema({
        vol.Required('request'): SCHEMA_VALIDATOR,
        vol.Required('response'): SCHEMA_VALIDATOR,
        vol.Optional('description'): str,
        vol.Optional('example'): str,
    })
})

ADVERTISEMENT_VALIDATOR = vol.Schema({
    vol.Required('provided_services'): PROVIDED_SERVICE_VALIDATOR,
    vol.Required('provided_topics'): {},
    vol.Required('expected_topics'): {},
})

INCOMING_MESSAGE_SCHEMA = vol.Any(
    vol.Schema({
        vol.Required('type'): 'call_service',
        vol.Required('instance_id'): int,
        vol.Required('domain'): str,
        vol.Required('name'): str,
        vol.Required('responds'): bool,
        vol.Required('request'): dict,
    }),
    vol.Schema({
        vol.Required('type'): 'respond_service',
        vol.Required('instance_id'): int,
        vol.Required('response'): vol.Any(
            dict,  # A proper response
            str,  # Some kind of error
        ),
    })
)


class ConnectionManager:
    def __init__(self, hass: HomeAssistant, host: str) -> None:
        self.hass = hass
        self.host = host
        self.next_service_call_instance_id = 0
        self.socket = None
        self.robot_configuration = None
        self.pending_outgoing_service_calls = {}
        asyncio.create_task(self.main_loop())

    async def _setup_configuration(self, configuration):
        service_info = configuration.get("provided_services")
        for service_name, service_info in service_info.items():
            request = service_info['request']
            response = service_info['response']

            async def handle_service_request(service_call: ServiceCall):
                if self.socket is not None:
                    # As long as we don't await between getting the instance
                    # id and incrementing the counter, this should be safe.
                    instance_id = self.next_service_call_instance_id
                    self.next_service_call_instance_id += 1

                    message = {
                        'type': 'call_service',
                        'instance_id': instance_id,
                        'service_name': service_name,
                        'request': service_call.data,
                    }

                    _LOGGER.info(f"Home Assistant Request: {message}")

                    future = asyncio.Future()
                    self.pending_outgoing_service_calls[instance_id] = \
                        future

                    message = cbor2.dumps(message)
                    await self.socket.send(message)
                    response = await future

                    match response:
                        case None:
                            # This indicates we got disconnected.
                            raise HomeAssistantError("Not connected to robot")
                        case dict():
                            # A successful response.
                            return response
                        case str():
                            # An error message.
                            raise HomeAssistantError(response)
                else:
                    raise HomeAssistantError("Not connected to robot")

            description = service_info.get(
                'description',
                f"Calls the `{service_name}` service on the robot")

            try:
                request_schema = decode_schema(
                    request)
                response_schema = decode_schema(
                    response)
                _LOGGER.info(
                    (f"Registered service `{service_name}` with scheme Request"
                     f"({request_schema}) "
                     f"Response({response_schema})"))

                supports_response = SupportsResponse.OPTIONAL

                fields = collect_metadata_for_fields(request)
                self.hass.services.async_register(
                    DOMAIN,
                    service_name,
                    handle_service_request,
                    schema=request_schema,
                    supports_response=supports_response)
                async_set_service_schema(
                    self.hass,
                    DOMAIN,
                    service_name,
                    {
                        "description": description,
                        "fields": fields,
                    })
            except (vol.error.Invalid, ValueError):
                _LOGGER.exception(
                    ("Failed to load layout for "
                     f"expected service {service_name}"))

        provided_topics = configuration.get("provided_topics")
        if provided_topics is not None:
            pass

        expected_topics = configuration.get("expected_topics")
        if expected_topics is not None:
            pass

    async def main_loop(self):
        # This loop handles connection failures.
        while True:
            try:
                # This loop handles clean disconnects.
                while True:
                    async with websockets.connect(self.host) as socket:
                        try:
                            self.socket = socket
                            await self.handle_socket_connection()
                        finally:
                            # Don't carry any pending service calls from a
                            # previous connection over. They're not going to
                            # get answered by this new instance.
                            for _instance_id, pending_future in self.\
                                    pending_outgoing_service_calls.items():
                                pending_future.set_result(None)
                            self.pending_outgoing_service_calls.clear()
                            self.socket = None
            except (InvalidURI,
                    OSError,
                    InvalidHandshake,
                    TimeoutError,
                    ConnectionClosedError) as e:
                _LOGGER.error(f"Websocket exception: {e}")
                await asyncio.sleep(DEFAULT_TIMEOUT_SECONDS)

    async def process_service_call_response(self, message):
        instance_id = message['instance_id']
        pending_future = self.pending_outgoing_service_calls.pop(
            instance_id, None)

        if pending_future:
            response = message['response']

            # The service may be an error string. Error handling will be done
            # in the service method, not here.
            pending_future.set_result(response)
        else:
            _LOGGER.warn(
                ("Received message response for "
                 f"non-pending message: {message}"))

    async def call_local_service(self, message):
        call_instance = message['instance_id']

        domain = message['domain']
        service_name = message['name']
        request = message['request']
        return_response = message['responds']

        _LOGGER.info(
            f"Call service `{service_name}` with request {request}")

        async def call_service():
            try:
                response = await self.hass.services.async_call(
                    domain, service_name, request, blocking=True,
                    return_response=return_response)

                response = {
                    'type': 'respond_service',
                    'instance_id': call_instance,
                    'response': response
                }
            except Exception as e:
                # async_call seems to be able to throw literally anything so we
                # have to catch anything.
                # Failures will be passed back to the caller.
                response = {
                    'type': 'respond_service',
                    'instance_id': call_instance,
                    'response': f"{e}"
                }

            _LOGGER.debug(f"RESPONSE: {response}")
            response = cbor2.dumps(response)
            if self.socket is not None:
                await self.socket.send(response)

        asyncio.create_task(call_service())

    async def handle_socket_connection(self):
        _LOGGER.info(
            f"Connected to {self.socket.remote_address}")

        try:
            try:
                configuration_advertisement = await self.socket.recv()
                configuration_advertisement = cbor2.loads(
                    configuration_advertisement)
                ADVERTISEMENT_VALIDATOR(configuration_advertisement)
            except CBORDecodeError:
                _LOGGER.exception(
                    "Failed to decode advertisement message")
                return
            except (vol.error.Invalid, ValueError):
                _LOGGER.exception(
                    "Failed to validate configuration advertisement")
                return
        finally:
            _LOGGER.info(
                f"Disconnected from {self.socket.remote_address}")

        await self._setup_configuration(configuration_advertisement)

        while True:
            try:
                message = await self.socket.recv()

                message = cbor2.loads(message)
                INCOMING_MESSAGE_SCHEMA(message)

                # Any other case should have been rejected by the
                # message schema.
                match message['type']:
                    case 'call_service':
                        await self.call_local_service(message)
                    case 'respond_service':
                        await self.process_service_call_response(message)

            except CBORDecodeError:
                _LOGGER.exception(
                    "Failed to decode incoming message")
            except (vol.error.Invalid, ValueError):
                _LOGGER.exception(
                    "Failed to validate incoming message")
            except ConnectionClosedOK:
                # Connection closed, break out of loop.
                break
