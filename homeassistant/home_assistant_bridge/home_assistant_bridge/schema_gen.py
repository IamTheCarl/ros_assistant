
from pyparsing import ParseException, Word, alphanums
from .class_importing import import_message_class
from .const import \
    INT8_MIN,  INT8_MAX, \
    INT16_MIN, INT16_MAX, \
    INT32_MIN, INT32_MAX, \
    INT64_MIN, INT64_MAX, \
 \
    UINT8_MIN,  UINT8_MAX, \
    UINT16_MIN, UINT16_MAX, \
    UINT32_MIN, UINT32_MAX, \
    UINT64_MIN, UINT64_MAX


PARSE_MESSAGE_PATH = \
    Word(alphanums + '_') + '/' + Word(alphanums + '_')

PARSE_SEQUENCE = \
    'sequence<' + Word(alphanums + '/_') + '>'


def generate_schema(message_class, logger):

    def encode_type(ty):
        match ty:
            case 'boolean':
                return {
                    'type': 'bool'
                }
            case 'octet':
                return {
                    'type': 'int',
                    'range': {
                        'min': UINT8_MIN,
                        'max': UINT8_MAX
                    }
                }

            case 'int8':
                return {
                    'type': 'int',
                    'range': {
                        'min': INT8_MIN,
                        'max': INT8_MAX
                    }
                }
            case 'int16':
                return {
                    'type': 'int',
                    'range': {
                        'min': INT16_MIN,
                        'max': INT16_MAX
                    }
                }
            case 'int32':
                return {
                    'type': 'int',
                    'range': {
                        'min': INT32_MIN,
                        'max': INT32_MAX
                    }
                }
            case 'int64':
                return {
                    'type': 'int',
                    'range': {
                        'min': INT64_MIN,
                        'max': INT64_MAX
                    }
                }

            case 'uint8':
                return {
                    'type': 'int',
                    'range': {
                        'min': UINT8_MIN,
                        'max': UINT8_MAX
                    }
                }
            case 'uint16':
                return {
                    'type': 'int',
                    'range': {
                        'min': UINT16_MIN,
                        'max': UINT16_MAX
                    }
                }
            case 'uint32':
                return {
                    'type': 'int',
                    'range': {
                        'min': UINT32_MIN,
                        'max': UINT32_MAX
                    }
                }
            case 'uint64':
                return {
                    'type': 'int',
                    'range': {
                        'min': UINT64_MIN,
                        'max': UINT64_MAX
                    }
                }

            case 'float':
                return {
                    'type': 'float'
                }
            case 'double':
                return {
                    'type': 'float',
                }

            case 'string':
                return {
                    'type': 'str'
                }

            # We're going to have to do more advanced parsing to figure
            # out what this is.
            case _:
                try:
                    [_sequence, subtype,
                        _closing_bracket] = PARSE_SEQUENCE.parse_string(ty)

                    return {
                        'type': 'sequence',
                        'subtype': encode_type(subtype)
                    }
                except ParseException:
                    # That failed to parse. It's not a sequence.
                    pass

                try:
                    [package, _slash,
                        message] = PARSE_MESSAGE_PATH.parse_string(ty)

                    message_class = import_message_class(
                        "msg", package, message, logger)

                    # None indicates we failed to import the class. We'll just
                    # leave it out of the schema. The `import_message_class`
                    # function will already have logged the event for us.
                    if message_class is not None:
                        return {
                            'type': 'dict',
                            'schema': generate_schema(message_class, logger)
                        }
                except ParseException:
                    # That failed to parse. It's not a message path.
                    pass

                logger.warn(f"Failed to parse field: {field}:{ty}")

    layout = message_class.get_fields_and_field_types()
    schema = {}

    for field, ty in layout.items():
        schema[field] = encode_type(ty)

    return schema
