
from home_assistant_bridge.schema_gen import generate_schema
from std_msgs import msg as msgs
from home_assistant_bridge.const import \
    INT8_MIN,  INT8_MAX, \
    INT16_MIN, INT16_MAX, \
    INT32_MIN, INT32_MAX, \
    INT64_MIN, INT64_MAX, \
 \
    UINT8_MIN,  UINT8_MAX, \
    UINT16_MIN, UINT16_MAX, \
    UINT32_MIN, UINT32_MAX, \
    UINT64_MIN, UINT64_MAX


def test_encode_layout_primitives():
    assert generate_schema(msgs.Empty, None) == {}
    assert generate_schema(msgs.Bool, None) == {
        'data': {
            'type': 'bool',
        }
    }
    assert generate_schema(msgs.Byte, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT8_MIN,
                'max': UINT8_MAX
            }
        }
    }
    assert generate_schema(msgs.Char, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT8_MIN,
                'max': UINT8_MAX
            }
        }
    }

    assert generate_schema(msgs.Int8, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': INT8_MIN,
                'max': INT8_MAX
            }
        }
    }
    assert generate_schema(msgs.Int16, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': INT16_MIN,
                'max': INT16_MAX
            }
        }
    }
    assert generate_schema(msgs.Int32, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': INT32_MIN,
                'max': INT32_MAX
            },
        }
    }
    assert generate_schema(msgs.Int64, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': INT64_MIN,
                'max': INT64_MAX
            },
        }
    }

    assert generate_schema(msgs.UInt8, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT8_MIN,
                'max': UINT8_MAX
            }
        }
    }
    assert generate_schema(msgs.UInt16, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT16_MIN,
                'max': UINT16_MAX
            }
        }
    }
    assert generate_schema(msgs.UInt32, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT32_MIN,
                'max': UINT32_MAX
            },
        }
    }
    assert generate_schema(msgs.UInt64, None) == {
        'data': {
            'type': 'int',
            'range': {
                'min': UINT64_MIN,
                'max': UINT64_MAX
            },
        }
    }

    assert generate_schema(msgs.Float32, None) == {
        'data': {
            'type': 'float'
        }
    }
    assert generate_schema(msgs.Float64, None) == {
        'data': {
            'type': 'float',
        }
    }

    assert generate_schema(msgs.String, None) == {
        'data': {
            'type': 'str'
        }
    }


def test_encode_layout_nested_dictionaries():
    assert generate_schema(msgs.Header, None) == {
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
    }


def test_encode_layout_sequences():
    assert generate_schema(msgs.UInt16MultiArray, None) == {
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
    }
