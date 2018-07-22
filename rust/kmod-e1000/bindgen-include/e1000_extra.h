
/* ICH GbE Flash Hardware Sequencing Flash Status Register bit breakdown */
/* Offset 04h HSFSTS */
union ich8_hws_flash_status {
	struct ich8_hsfsts {
		u16 flcdone:1; /* bit 0 Flash Cycle Done */
		u16 flcerr:1; /* bit 1 Flash Cycle Error */
		u16 dael:1; /* bit 2 Direct Access error Log */
		u16 berasesz:2; /* bit 4:3 Sector Erase Size */
		u16 flcinprog:1; /* bit 5 flash cycle in Progress */
		u16 reserved1:2; /* bit 13:6 Reserved */
		u16 reserved2:6; /* bit 13:6 Reserved */
		u16 fldesvalid:1; /* bit 14 Flash Descriptor Valid */
		u16 flockdn:1; /* bit 15 Flash Config Lock-Down */
	} hsf_status;
	u16 regval;
};

/* ICH GbE Flash Hardware Sequencing Flash control Register bit breakdown */
/* Offset 06h FLCTL */
union ich8_hws_flash_ctrl {
	struct ich8_hsflctl {
		u16 flcgo:1;   /* 0 Flash Cycle Go */
		u16 flcycle:2;   /* 2:1 Flash Cycle */
		u16 reserved:5;   /* 7:3 Reserved  */
		u16 fldbcount:2;   /* 9:8 Flash Data Byte Count */
		u16 flockdn:6;   /* 15:10 Reserved */
	} hsf_ctrl;
	u16 regval;
};

/* ICH Flash Region Access Permissions */
union ich8_hws_flash_regacc {
	struct ich8_flracc {
		u32 grra:8; /* 0:7 GbE region Read Access */
		u32 grwa:8; /* 8:15 GbE region Write Access */
		u32 gmrag:8; /* 23:16 GbE Master Read Access Grant */
		u32 gmwag:8; /* 31:24 GbE Master Write Access Grant */
	} hsf_flregacc;
	u16 regval;
};
