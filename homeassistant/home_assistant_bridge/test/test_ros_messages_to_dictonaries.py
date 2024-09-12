
from home_assistant_bridge.ros_messages_to_dictionaries \
    import ros_message_to_dictionary
from std_msgs import msg as msgs
from builtin_interfaces import msg as builtin_msgs


def test_primitives():
    assert ros_message_to_dictionary(msgs.Empty()) == {}
    assert ros_message_to_dictionary(msgs.Bool(data=True)) == {'data': True}
    assert ros_message_to_dictionary(msgs.Byte(data=bytes([24]))) == {
        'data': bytes([24])}
    assert ros_message_to_dictionary(msgs.Char(data=25)) == {'data': 25}

    assert ros_message_to_dictionary(msgs.Int8(data=26)) == {'data': 26}
    assert ros_message_to_dictionary(msgs.Int16(data=27)) == {'data': 27}
    assert ros_message_to_dictionary(msgs.Int32(data=28)) == {'data': 28}
    assert ros_message_to_dictionary(msgs.Int64(data=29)) == {'data': 29}

    assert ros_message_to_dictionary(msgs.UInt8(data=30)) == {'data': 30}
    assert ros_message_to_dictionary(msgs.UInt16(data=31)) == {'data': 31}
    assert ros_message_to_dictionary(msgs.UInt32(data=32)) == {'data': 32}
    assert ros_message_to_dictionary(msgs.UInt64(data=33)) == {'data': 33}

    assert ros_message_to_dictionary(msgs.Float32(data=34.0)) == {'data': 34.0}
    assert ros_message_to_dictionary(msgs.Float64(data=35.0)) == {'data': 35.0}

    assert ros_message_to_dictionary(
        msgs.String(data='Some text')) == {'data': 'Some text'}


def test_nested_dictionaries():
    assert ros_message_to_dictionary(msgs.Header(
        stamp=builtin_msgs.Time(
            sec=1,
            nanosec=2),
        frame_id='my_frame')) == {
        'stamp': {
            'sec': 1,
            'nanosec': 2,
        },
        'frame_id': 'my_frame'
    }


def test_sequences():
    assert ros_message_to_dictionary(msgs.UInt16MultiArray(
        layout=msgs.MultiArrayLayout(
            dim=[msgs.MultiArrayDimension(
                label='a',
                size=32,
                stride=23)],
            data_offset=42),
        data=[12, 34, 56])) == {
        'layout': {
            'dim': [
                {
                    'label': 'a',
                    'size': 32,
                    'stride': 23
                }
            ],
            'data_offset': 42
        },
        'data': [12, 34, 56]
    }
