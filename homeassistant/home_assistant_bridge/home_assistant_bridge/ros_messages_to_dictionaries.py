
from .schema_gen import PARSE_MESSAGE_PATH, PARSE_SEQUENCE
from pyparsing import ParseException


def convert_type(ty, value):
    match ty:
        # Primitives can just be directly copied over.
        case 'boolean' | 'octet' | 'byte' | 'char' | 'string' | \
                'float' | 'double' | \
                'int8' | 'int16' | 'int32' | 'int64' | \
                'uint8' | 'uint16' | 'uint32' | 'uint64':
            return value
        case _:
            # It's a sequence or a more advanced structure.
            try:
                [_sequence, subtype,
                    _closing_bracket] = PARSE_SEQUENCE.parse_string(ty)

                array = [convert_type(subtype, sub_value)
                         for sub_value in value]

                return array
            except ParseException as e1:
                # That failed to parse. It's not a sequence.
                try:
                    # This is a message structure.
                    [_package, _slash,
                        _class_name] = PARSE_MESSAGE_PATH.parse_string(ty)

                    return ros_message_to_dictionary(value)
                except ParseException as e2:
                    raise (e1, e2)


def ros_message_to_dictionary(message):
    dictionary = {}
    for field, ty in message.get_fields_and_field_types().items():
        dictionary[field] = convert_type(ty, getattr(message, field))

    return dictionary
