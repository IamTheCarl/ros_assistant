
import voluptuous as vol

SCHEMA_VALIDATOR = vol.Schema({}, extra=vol.ALLOW_EXTRA)
FIELD_VALIDATOR = vol.Schema(vol.Any(
    vol.Schema({
        vol.Required('type'): vol.Any('bool', 'str'),
    }, extra=vol.ALLOW_EXTRA),
    vol.Schema({
        vol.Required('type'): vol.Any('int', 'float'),
        vol.Optional('range'): vol.Schema({
            vol.Optional('min'): vol.Any(int, float),
            vol.Optional('max'): vol.Any(int, float),
        }),
    }, extra=vol.ALLOW_EXTRA),
    vol.Schema({
        vol.Required('type'): 'dict',
        vol.Required('schema'): SCHEMA_VALIDATOR,
    }, extra=vol.ALLOW_EXTRA),
    vol.Schema({
        vol.Required('type'): 'sequence',
        vol.Required('subtype'): vol.Self,
    }, extra=vol.ALLOW_EXTRA)
))

SCHEMA_VALIDATOR.extend({
    str: FIELD_VALIDATOR
})


def _decode_range(range):
    minimum = range.get('min')
    maximum = range.get('max')

    if minimum is not None or maximum is not None:
        return vol.Range(min=minimum, max=maximum)
    else:
        return None


def _combine_constraints(*constraints):
    constraints = [
        constraint for constraint in constraints if constraint is not None]

    if len(constraints) <= 1:
        return constraints[0]
    else:
        return vol.All(*constraints)


def _decode_field(value):
    ty = value['type']

    match ty:
        case 'bool':
            return bool
        case 'int':
            range = value.get('range')
            if range is not None:
                range = _decode_range(range)

            return _combine_constraints(int, range)
        case 'float':
            range = value.get('range')
            if range is not None:
                range = _decode_range(range)

            return _combine_constraints(float, range)
        case 'str':
            return str
        case 'dict':
            return _inner_decode_schema(value['schema'])
        case 'sequence':
            subtype = _decode_field(value['subtype'])
            return [subtype]
        case _:
            raise ValueError(
                f"Unknown type: {value}")

# Decodes a schema, but without the struct validation.


def _inner_decode_schema(input):
    schema = {}

    for member, value in input.items():
        value = _decode_field(value)
        schema[vol.Required(member)] = value

    return vol.Schema(schema)


def decode_schema(schema):
    SCHEMA_VALIDATOR(schema)
    return _inner_decode_schema(schema)
