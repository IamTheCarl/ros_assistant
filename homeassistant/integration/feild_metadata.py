
from .schema_decode import SCHEMA_VALIDATOR
from homeassistant.const import CONF_MODE
import logging

_LOGGER = logging.getLogger(__name__)


def collect_metadata_for_fields(schema):
    SCHEMA_VALIDATOR(schema)

    fields = {}
    for field_name, field in schema.items():
        fields[field_name] = {
            'name': field_name,
            'required': True,
            'description': field.get(
                'description', ''),
            'example': field.get('example', ''),
            'selector': _get_selector_for_field_type(field['type']),
        }

    return fields


def _get_selector_for_field_type(type_name):
    match type_name:
        case 'bool':
            return {'boolean': None}
        case 'int':
            return {'number': {CONF_MODE: 'box'}}
        case 'float':
            return {'number': {CONF_MODE: 'box', "step": 1e-3}}
        case 'str':
            return {'text': {}}
        case _:
            return {'object': {}}
