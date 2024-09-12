
from home_assistant_bridge.dictionaries_to_ros_messages \
    import dictionary_to_ros_message
from std_msgs import msg as msgs
from builtin_interfaces import msg as builtin_msgs
import logging

_LOGGER = logging.getLogger()


def test_primitives():
    assert dictionary_to_ros_message(msgs.Empty, {}, _LOGGER) == msgs.Empty()
    assert dictionary_to_ros_message(
        msgs.Bool, {'data': True}, _LOGGER) == msgs.Bool(data=True)
    assert dictionary_to_ros_message(
        msgs.Byte, {'data': bytes([24])},
        _LOGGER) == msgs.Byte(data=bytes([24]))
    assert dictionary_to_ros_message(
        msgs.Char, {'data': 25}, _LOGGER) == msgs.Char(data=25)

    assert dictionary_to_ros_message(
        msgs.Int8, {'data': 26}, _LOGGER) == msgs.Int8(data=26)
    assert dictionary_to_ros_message(
        msgs.Int16, {'data': 27}, _LOGGER) == msgs.Int16(data=27)
    assert dictionary_to_ros_message(
        msgs.Int32, {'data': 28}, _LOGGER) == msgs.Int32(data=28)
    assert dictionary_to_ros_message(
        msgs.Int64, {'data': 29}, _LOGGER) == msgs.Int64(data=29)

    assert dictionary_to_ros_message(
        msgs.UInt8, {'data': 30}, _LOGGER) == msgs.UInt8(data=30)
    assert dictionary_to_ros_message(
        msgs.UInt16, {'data': 31}, _LOGGER) == msgs.UInt16(data=31)
    assert dictionary_to_ros_message(
        msgs.UInt32, {'data': 32}, _LOGGER) == msgs.UInt32(data=32)
    assert dictionary_to_ros_message(
        msgs.UInt64, {'data': 33}, _LOGGER) == msgs.UInt64(data=33)

    assert dictionary_to_ros_message(
        msgs.Float32, {'data': 34.0}, _LOGGER) == msgs.Float32(data=34.0)
    assert dictionary_to_ros_message(
        msgs.Float64, {'data': 35.0}, _LOGGER) == msgs.Float64(data=35.0)

    assert dictionary_to_ros_message(
        msgs.String, {'data': 'Some text'},
        _LOGGER) == msgs.String(data='Some text')


def test_nested_dictionaries():
    assert dictionary_to_ros_message(msgs.Header, {
        'stamp': {
            'sec': 1,
            'nanosec': 2,
        },
        'frame_id': 'my_frame'
    }, _LOGGER) == msgs.Header(
        stamp=builtin_msgs.Time(
            sec=1,
            nanosec=2),
        frame_id='my_frame')


def test_sequences():
    assert dictionary_to_ros_message(msgs.UInt16MultiArray, {
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
    }, _LOGGER) == msgs.UInt16MultiArray(
        layout=msgs.MultiArrayLayout(
            dim=[msgs.MultiArrayDimension(
                label='a',
                size=32,
                stride=23)],
            data_offset=42),
        data=[12, 34, 56])
