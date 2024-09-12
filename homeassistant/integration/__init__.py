from __future__ import annotations

import asyncio

from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant

from .connection_manager import ConnectionManager
from .const import CONF_HOST

async def async_setup_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    entry.runtime_data = ConnectionManager(hass, entry.data[CONF_HOST])

    return True