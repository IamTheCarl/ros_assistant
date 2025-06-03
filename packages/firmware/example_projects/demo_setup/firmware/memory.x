MEMORY
{
    /* Total of 256K of flash */
    FLASH            : ORIGIN = 0x08000000, LENGTH =  256K /* BANK_1 */
    BOOTLOADER_STATE : ORIGIN = 0x08006000, LENGTH = 8K
    ACTIVE           : ORIGIN = 0x08008000, LENGTH = 122K
    DFU              : ORIGIN = 0x08010000, LENGTH = 126K
    RAM              : ORIGIN = 0x20000000, LENGTH =   40K /* SRAM */
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(FLASH);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(FLASH);

