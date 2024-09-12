
from .class_importing import import_message_class
from .schema_gen import PARSE_MESSAGE_PATH, PARSE_SEQUENCE
from pyparsing import ParseException


def evaluate_type(ty, value, logger):
    match ty:
        case None:
            # We couldn't figure out the type for this, so we'll just
            # pipe it straight over. The ROS code will throw an exception
            # if appropriate.
            return value
        case 'boolean' | 'octet' | 'byte' | 'char' | 'string' | \
            'float' | 'double' | \
            'int8' | 'int16' | 'int32' | 'int64' | \
                'uint8' | 'uint16' | 'uint32' | 'uint64':
            return value
        case _:

            try:
                [_sequence, subtype,
                    _closing_bracket] = PARSE_SEQUENCE.parse_string(ty)

                array = [evaluate_type(subtype, sub_value, logger)
                         for sub_value in value]

                return array
            except ParseException as e1:
                # That failed to parse. It's not a sequence.
                try:
                    # This is a message structure.
                    [package, _slash,
                        class_name] = PARSE_MESSAGE_PATH.parse_string(ty)

                    sub_message_class = import_message_class(
                        'msg', package, class_name, logger)

                    return dictionary_to_ros_message(
                        sub_message_class, value, logger)
                except ParseException as e2:
                    raise (e1, e2)


def dictionary_to_ros_message(message_class, dictionary, logger):
    constructed_dictionary = {}

    type_dictionary = message_class.get_fields_and_field_types()

    for field_name, value in dictionary.items():
        ty = type_dictionary.get(field_name)
        constructed_dictionary[field_name] = evaluate_type(
            ty, value, logger)

    return message_class(**constructed_dictionary)
