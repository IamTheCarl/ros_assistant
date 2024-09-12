
from deepdiff import DeepDiff
import voluptuous as vol
from integration.schema_decode import decode_schema
from integration.const import \
    INT32_MIN, INT32_MAX, \
    UINT32_MIN, UINT32_MAX, \
    UINT16_MIN, UINT16_MAX, \
    INT64_MAX


def test_decode_layout_primitives():
    assert not DeepDiff(decode_schema({}), vol.Schema({}))
    assert not DeepDiff(decode_schema({
        'data': {
            'type': 'bool',
        }
    }), vol.Schema({
        vol.Required('data'): bool
    }))

    assert not DeepDiff(decode_schema({
        'data': {
            'type': 'int',
            'range': {
                'min': INT32_MIN,
                'max': INT64_MAX
            }
        }
    }), vol.Schema({
        vol.Required('data'):
            vol.All(int, vol.Range(min=INT32_MIN, max=INT64_MAX)),
    }))
    assert not DeepDiff(decode_schema({
        'data': {
            'type': 'float',
        }
    }), vol.Schema({
        vol.Required('data'): float,
    }))
    assert not DeepDiff(decode_schema({
        'data': {
            'type': 'str',
        }
    }), vol.Schema({
        vol.Required('data'): str,
    }))


def test_decode_layout_nested_dictionaries():
    assert not DeepDiff(decode_schema({
        'stamp': {
            'type': 'dict',
            'schema': {
                'sec': {
                    'type': 'int',
                    'range': {
                        'min': INT32_MIN,
                        'max': INT32_MAX
                    }
                },
                'nanosec': {
                    'type': 'int',
                    'range': {
                        'min': UINT32_MIN,
                        'max': UINT32_MAX
                    }
                }
            }
        },
        'frame_id': {
            'type': 'str'
        }
    }), vol.Schema({
        vol.Required('stamp'): vol.Schema({
            vol.Required('sec'):
                vol.All(int, vol.Range(min=INT32_MIN, max=INT32_MAX)),
            vol.Required('nanosec'):
                vol.All(int, vol.Range(min=UINT32_MIN, max=UINT32_MAX))
        }),
        vol.Required('frame_id'): str
    }))


def test_decode_layout_sequences():
    assert not DeepDiff(decode_schema({
        'layout': {
            'type': 'dict',
            'schema': {
                'dim': {
                    'type': 'sequence',
                    'subtype': {
                        'type': 'dict',
                        'schema': {
                            'label': {
                                'type': 'str'
                            },
                            'size': {
                                'type': 'int',
                                'range': {
                                    'min': UINT32_MIN,
                                    'max': UINT32_MAX,
                                }
                            },
                            'stride': {
                                'type': 'int',
                                'range': {
                                    'min': UINT32_MIN,
                                    'max': UINT32_MAX
                                }
                            }
                        }
                    }
                },
                'data_offset': {
                    'type': 'int',
                    'range': {
                        'min': UINT32_MIN,
                        'max': UINT32_MAX
                    }
                }
            }
        },
        'data': {
            'type': 'sequence',
            'subtype': {
                'type': 'int',
                'range': {
                    'min': UINT16_MIN,
                    'max': UINT16_MAX
                }
            }
        }
    }), vol.Schema({
        vol.Required('layout'): vol.Schema({
            vol.Required('dim'): [vol.Schema({
                vol.Required('label'): str,
                vol.Required('size'):
                    vol.All(int, vol.Range(min=UINT32_MIN, max=UINT32_MAX)),
                vol.Required('stride'):
                    vol.All(int, vol.Range(min=UINT32_MIN, max=UINT32_MAX))
            })],
            vol.Required('data_offset'):
                vol.All(int, vol.Range(min=UINT32_MIN, max=UINT32_MAX)),
        }),
        vol.Required('data'):
            [vol.All(int, vol.Range(min=UINT16_MIN, max=UINT16_MAX))]
    }))
