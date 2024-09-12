
import importlib


def import_message_class(prefix, package, message_name, logger):
    module_path = f"{package}.{prefix}"

    try:
        module = importlib.import_module(module_path)
    except ImportError as e:
        logger.error(f"Failed to import `{message_name}`: {e}")

    try:
        if hasattr(module, message_name):
            return getattr(module, message_name)
        else:
            logger \
                .error((f"Failed to import `{package}/{message_name}`: "
                        "The module was successfully imported but it did not "
                        f"contain the message `{message_name}`"))
            return None
    except UnboundLocalError:
        logger \
            .error((f"Failed to import `{package}/{message_name}`: "
                    "The module does not exist."))
        return None
