from homeassistant import config_entries
import voluptuous as vol
from .const import DOMAIN, CONF_NAME, CONF_HOST
import logging
from urllib.parse import urlparse

_LOGGER = logging.getLogger(__name__)

DATA_SCHEMA = vol.Schema({
    vol.Required(CONF_NAME): str,
    vol.Required(CONF_HOST): str,
})


def validate_url(url):
    try:
        result = urlparse(url)
        if result.scheme == "ws":
            return (True, None)
        else:
            return (False, "invalid_uri")
    except AttributeError:
        return (False, "invalid_uri")


class ExampleConfigFlow(config_entries.ConfigFlow, domain=DOMAIN):
    VERSION = 1
    MINOR_VERSION = 0
    CONNECTION_CLASS = config_entries.CONN_CLASS_LOCAL_PUSH

    async def async_step_user(self, user_input=None):
        errors = {}
        if user_input is not None:
            (valid, error) = validate_url(user_input[CONF_HOST])
            if valid:
                return self.async_create_entry(
                    title=user_input[CONF_NAME],
                    data=user_input)
            else:
                errors[CONF_HOST] = error

        return self.async_show_form(
            step_id="user",
            data_schema=DATA_SCHEMA,
            errors=errors)
