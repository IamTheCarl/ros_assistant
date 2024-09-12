
from home_assistant_bridge.class_importing import import_message_class
import std_msgs.msg as msg
import std_srvs.srv as srv
import logging

_LOGGER = logging.getLogger(__name__)


def test_dynamic_message_import():
    assert import_message_class(
        "msg", "std_msgs", "Int16", _LOGGER) is msg.Int16


def test_dynamic_service_import():
    assert import_message_class(
        "srv", "std_srvs", "SetBool", _LOGGER) is srv.SetBool


def test_import_fail():
    assert import_message_class("srv", "doesn't", "exist", _LOGGER) is None
    assert import_message_class("srv", "std_srvs", "exist", _LOGGER) is None
