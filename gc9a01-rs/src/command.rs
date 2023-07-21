//! Commands

use display_interface::{DataFormat::U8, DisplayError, WriteOnlyDataCommand};

/// GC9A01 Commands
#[derive(Debug, Copy, Clone)]
pub enum Command {
    /// Set Sleep mode (10h/11h)
    ///
    /// This command turns on/off sleep mode.
    ///
    /// ## Description
    ///
    /// This command causes the LCD module to enter the minimum power consumption mode.
    /// In this mode e.g. the DC/DC converter is stopped, Internal oscillator is stopped, and panel scanning is stopped
    ///
    /// ## Restriction
    ///
    /// ## On Logical On (10h)
    ///
    /// It will be necessary to wait 5msec before sending next to command,
    /// this is to allow time for the supply voltages and clock circuits to
    /// stabilize.
    ///
    /// ## On Logical Off (11h)
    ///
    /// It will be necessary to wait 120msec after sending Sleep Out command (when in
    /// Sleep In Mode) before Sleep In command can be sent.
    ///
    SleepMode(Logical),

    /// Set Partial mode (12h)
    ///
    /// This command turns on Partial mode.
    ///
    /// ## Description
    ///
    /// This command turns on partial mode. The Partial mode window is described by the Partial
    /// Area command (30h). To leave Partial mode, the Normal Display Mode On command (13h) should be written.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when Partial mode is active.
    ///
    PartialMode,

    /// Set Normal Display mode (13h)
    ///
    /// This command turns on Normal Display mode.
    ///
    /// ## Description
    ///
    /// Normal display mode on means Partial mode off.
    /// Exit from NORON by the Partial mode On command (12h)
    ///
    /// ## Restriction
    ///
    /// This command has no effect when Normal Display mode is active.
    ///
    NormalDisplayMode,

    /// Set Display Inversion (20h/21h)
    ///
    /// This command turns on/off Display Inversion
    ///
    /// ## Description
    ///
    /// This command is used to recover from display inversion mode.
    /// This command makes no change of the content of frame memory.
    /// This command doesn't change any other status.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when module already is inversion OFF mode.
    ///
    DisplayInversion(Logical),

    /// Set Display State (28h/29h)
    ///
    /// This command turns on/off Display
    ///
    /// ## Description
    ///
    /// In this mode, the output from Frame Memory is disabled and blank page inserted.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when module is already in display OFF mode.
    ///
    DisplayState(Logical),

    /// Set Column Address (start, end) (2Ah)
    ///
    /// ## Parameters
    ///
    ///  * SC `.0` => Start Column
    /// * EC `.1` => End Column
    ///
    /// This command is used to defined area of frame memory where MCU can access.
    ///
    /// ## Description
    ///
    /// This command makes no change on the other driver status.
    /// The values of SC [15:0] and EC [15:0] are referred when RAMWR command comes.
    /// Each value represents one column line in the frame memory.
    ///
    /// ## Restriction
    ///
    /// SC [15:0] always must be equal to or less than EC [15:0].
    ///
    /// __NOTE__: When SC [15:0] or EC [15:0] is greater than 013Fh (When MADCTL's B5 = 0)
    /// or 00EFh (When MADCTL's B5 = 1), data of out of range will be ignored.
    ///
    ColumnAddressSet(u16, u16),

    /// Set Row Address (start, end) (2Bh)
    ///
    /// ## Parameters
    ///
    ///  * SP `.0` => Start Page/Row
    /// * EP `.1` => End Page/Row
    ///
    /// This command is used to define area of frame memory where MCU can access.
    ///
    /// ## Description
    ///
    /// This command makes no change on the other driver status.
    /// The values of SP [15:0] and EP [15:0] are referred when RAMWR command comes.
    /// Each value represents one Page line in the Frame Memory
    ///
    /// ## Restriction
    ///
    /// SP [15:0] always must be equal to or less than EP [15:0]
    ///
    /// __NOTE__:When SP [15:0] or EP [15:0] is greater than 00EFh (When MADCTL’s B5 = 0)
    /// or 013Fh (When MADCTL’s B5 = 1), data of out of range will be ignored.
    ///
    RowAddressSet(u16, u16),
    // Partial Area (start, end) (30)
    // TODO:>
    // TODO: PartialArea(u16, u16),
    //
    /// Vertical Scrolling Definition (33h)
    ///
    /// ## Parameters
    ///
    /// * TFA `.0` => Top Fixed Area
    /// * VSA `.1` => Vertical Scrolling Area
    ///
    /// This command is used to define area of frame memory where MCU can access.
    ///
    /// ## Description
    ///
    /// This command defines the Vertical Scrolling Area of the display.
    ///
    /// ## When MADCTL B4=0
    ///
    /// The 1st & 2nd parameter TFA [15...0] describes the Top Fixed Area (in No. of lines from
    /// Top of the Frame Memory and Display).
    /// The 3rd & 4th parameter VSA [15...0] describes the height of the Vertical Scrolling Area
    /// (in No. of lines of the Frame Memory [not the display]
    /// from the Vertical Scrolling Start Address).
    /// The first line read from Frame Memory appears immediately after the bottom most line of the Top Fixed Area.
    ///
    /// ## When MADCTL B4=1
    ///
    /// The 1st & 2nd parameter TFA [15...0] describes the Top Fixed Area (in No. of lines from
    /// Bottom of the Frame Memory and Display).
    /// The 3rd & 4th parameter VSA [15...0] describes the height of the Vertical Scrolling Area
    /// (in No. of lines of the Frame
    /// Memory [not the display] from the Vertical Scrolling Start Address). The first line read
    /// from Frame Memory appears
    /// immediately after the top most line of the Top Fixed Area
    ///
    VertialScrollDef(u16, u16),

    /// Tearing Effect Line OFF (35h)
    /// Tearing Effect Line OFF (34h)
    ///
    /// This command turns on tearing effect line with a parameters.
    ///
    /// ## Parameters
    ///
    /// * M `.0` => Mode (Logical)
    ///
    /// ## Description
    ///
    /// This command is used to turn ON the Tearing Effect output signal from the TE signal line.
    /// This output is not affected by changing MADCTL bit B4. The Tearing Effect Line On has one parameter which describes
    /// the mode of the Tearing Effect Output Line.
    ///
    /// This command is used to turn OFF
    /// (Active Low) the Tearing Effect output signal from the TE signal line.
    ///
    ///
    /// ## Restriction
    ///
    /// This command has no effect when Tearing Effect output is already ON
    ///
    TearingEffectLine(Logical),

    /// Memory Access Control (36h)
    ///
    /// This command defines read/write scanning direction of frame memory.
    /// This command makes no change on the other driver status
    /// ## Parameters
    ///
    /// * MY Row Address Order
    /// * MX Column Address Order These 3 bits control MCU to memory write/read direction.
    /// * MV Row / Column Exchange
    /// * ML Vertical Refresh Order LCD vertical refresh direction control.
    /// * BGR RGB-BGR Order
    /// * Color selector switch control (0=RGB color filter panel, 1=BGR color filter panel)
    /// * MH Horizontal Refresh ORDER LCD horizontal refreshing direction control.
    ///
    /// ## Description
    ///
    /// Note: When BGR bit is changed, the new setting is active immediately without update the content in
    /// Frame Memory again.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when Tearing Effect output is already ON
    ///
    MemoryAccessControl(Logical, Logical, Logical, Logical, Logical, Logical),

    /// Vertical Scrolling Start Address (37h)
    ///
    /// ## Parameters
    /// * VSP `.0` => Vertical Start Page
    ///
    /// ## Description
    ///
    /// This command is used together with Vertical Scrolling Definition (33h). These two commands
    /// describe the scrolling area and the scrolling mode.
    ///
    /// The Vertical Scrolling Start Address command has one parameter
    /// which describes the address of the line in the Frame Memory that will be written as the first line after
    /// the last line of the Top Fixed Area (TFA) on the display.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when Tearing Effect output is already ON.
    ///
    VerticalScrollStartAddresss(u16),

    /// Set Idle Mode (38h/39h)
    ///
    /// This command turns on/off Idle Mode
    ///
    /// ## Parameters
    /// * State `.0` => On/Off
    ///
    /// ## Description
    ///
    /// In the idle off mode, LCD can display maximum 262,144 colors.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when module is already in idle same mode.
    ///
    IdleMode(Logical),

    /// Pixel Format Set (3Ah) COLMOD
    ///
    /// ## Parameters
    /// * DBI `.0` => MCU Interface Format
    /// * DPI `.1` => RGB Interface Format
    ///
    /// ## Description
    ///
    /// This command sets the pixel format for the RGB image data used by the interface. DPI [2:0] is
    /// the pixel format select of RGB interface and DBI [2:0] is the pixel format of MCU interface. If
    /// a particular interface, either RGB interface or MCU interface, is not used then the
    /// corresponding bits in the parameter are ignored. The pixel format is shown in the table below.
    ///
    /// ## Restriction
    ///
    /// This command has no effect when module is already in idle off mode.
    PixelFormatSet(Dbi, Dpi),

    /// Set Tear Scanline (44h)
    ///
    /// ## Parameters
    /// * STS => Set Tear Scanline 0-239
    ///
    /// ## Description
    ///
    /// This command turns on the display Tearing Effect output signal on the TE signal line when the
    /// display reaches line equal the value of STS[8:0].
    ///
    /// __NOTE__: that set_tear_scanline with STS is equivalent to set_tear_on with 8+GateN(N=1、2、3...240)
    SetTearScanline(u16),

    /// Write Display Brightness (51h)
    ///
    /// This command is used to adjust the brightness value of the display
    ///
    /// ## Parameters
    ///
    /// * DBV `.0` =>
    ///
    /// ## Description
    ///
    /// It should be checked what is the relationship between this written value and output brightness of the display.
    /// This relationship between this written value and output brightness of the display. This relationship is defined on the display module specification.
    /// In principle relationship is that 00h value means the lowest brightness and FFh value means the highest brightness.
    ///
    /// ## Math
    ///
    /// * DBV[7:0]/255 x period (affected by OSC frequency)
    ///
    /// For example: LEDPWM period = 3ms, and DBV[7:0] = ‘200DEC’. Then LEDPWM duty = 200 / 255=78.1%.
    /// Correspond to the LEDPWM period = 3 ms, the high-level of LEDPWM (high effective) = 2.344ms, and the
    /// low-level of LEDPWM = 0.656ms.
    ///
    DisplayBrightness(u8),

    /// Write CTRL Display (53h)
    ///
    /// ## Parameters
    ///
    /// * BCTRL `.0` => Brightness Control Block On/Off
    /// * DD `.1` => Display Dimming On/Off
    /// * BL `.2` => Backlight On/Off
    ///
    /// ## Restriction
    ///
    /// The display module is sending 2nd parameter value on the data line if the MCU
    /// wants to read more than one parameters
    /// (=more than 2 RDX Cyle) on DBI
    /// Only 2nd parameter is sent on DSI (The 1st parameter is not sent).
    CtrlDisplay(Logical, Logical, Logical),

    /// RGB Interface Signal Control (B0h)
    ///
    /// ## Parameters
    ///
    /// * EPL `.0` => DE polarity (“0”= High enable for RGB interface, “1”= Low enable for RGB interface)
    /// * DP `.1` => DOTCLK polarity set (“0”= data fetched at the rising time, “1”= data fetched at the falling time)
    /// * HSP `.2` => HSYNC polarity (“0”= Low level sync clock, “1”= High level sync clock)
    /// * VSP `.3` => VSYNC polarity (“0”= Low level sync clock, “1”= High level sync clock)
    /// * RCM `.4` => RGB interface selection (refer to the RGB interface section).
    ///
    RGBInterfaceSignalCtrl(DEPolarity, DOTClk, XSpl, XSpl, RCMMode),

    /// Blanking Porch Control (B5h)
    ///
    /// ## Parameters
    ///
    /// * VFP `.0` => The line number of Vertical Front Porch
    /// * VBP `.1` => The line number of Vertical Back Porch
    /// * HBP `.2` => The line number of Horizontal Back Porch
    ///
    /// ## Description
    ///
    /// __NOTE__: The Third parameter must write,but it is not valid
    ///
    BlankingPorchControl(u8, u8, u8),

    /// Display Function Control (B6h)
    ///
    /// ## Parameters
    ///
    /// * GS `.0` => Sets the direction of scan by the gate driver in the range determined by SCN [4:0]
    /// * SS `.1` => Select the shift direction of outputs from the source driver.
    /// * SM `.2` => Sets the gate driver pin arrangement in combination with the GS bit to select the optimal scan mode for the module
    /// * NL `.3` =>  Sets the number of lines to drive the LCD at an interval of 8 lines.
    ///
    /// ## Restriction
    ///
    /// EXTC should be high to enable this command
    ///
    DispalyFunctionControl(GSMode, SSMode, u8, u8),

    /// TE Control (BAh)
    ///
    /// ## Parameters
    ///
    /// * te_pol `.0` => [`TEPolarity`] is used to adjust the Tearing Effect output signal pulse polarity
    /// * te_width `.1` => TODO
    ///
    /// ## Restriction
    ///
    /// __NOTE__: During Sleep In Mode with Tearing Effect Line On, Tearing Effect Output pin will be active Low
    /// This command has no effect when Tearing Effect output is already ON
    ///
    TEControl(TEPolarity, u8),

    /// Interface Control (F6h)
    ///
    /// ## Parameters
    ///
    /// * DM `.0` => TODO
    /// * RM `.1` => TODO
    /// * RIM `.2` => TODO
    ///
    /// ## Restriction
    ///
    /// EXTC should be high to enable this command
    ///
    Interface(DMMode, RMMode, RIMMode),

    /// Power Criterion Control (C1h)
    ///
    /// ## Parameters
    ///
    /// * vcire `.0` => TODO
    ///
    PowerCriterioControl(VCIRe),

    /// VCore Voltage Control (A7h)
    ///
    /// ## Parameters
    ///
    /// * vdd_ad `.0` => TODO
    ///
    VCoreVoltageControl(VddAd),

    /// Vreg 1a Voltage Control (C3h)
    ///
    /// ## Parameters
    ///
    /// * vreg1_vbp_d `.0` => TODO
    ///
    /// ## Description
    ///
    /// Set the voltage level value to output the VREG1A and VREG1B OUT level, which is a
    /// reference level for the grayscale voltage level.(Table is valid when vrh=0x28)
    /// VREG1A=(vrh+vbp_d)*0.02+4
    /// VREG1B=vbp_d*0.02+0.3
    ///
    Vreg1aVoltageControl(u8),

    /// Vreg 1b Voltage Control (C4h)
    ///
    /// ## Parameters
    ///
    /// * vreg1_vbn_d `.0` => TODO
    ///
    /// ## Description
    ///
    /// Set the voltage level value to output the VREG2A OUT level, which is a reference level for
    /// the grayscale voltage level(Table is valid when vrh=0x28)
    /// VREG2A=(vbn_d-vrh)*0.02-3.4
    /// VREG2B=vbn_d*0.02+0.3
    ///
    Vreg1bVoltageControl(u8),

    /// Vreg 2a Voltage Control (C9h)
    ///
    /// ## Parameters
    ///
    /// * vrh `.0` => TODO
    ///
    /// ## Description
    ///
    /// Set the voltage level value to output the VREG1A OUT level, which is a reference level for
    /// the grayscale voltage level. (Table is valid when vbp_d=0x3C and vbn_d=0x3C)
    /// VREG1A=(vrh+vbp_d)*0.02+4
    /// VREG2A=(vbn_d-vrh)*0.02-3.4
    ///
    Vreg2aVoltageControl(u8),

    /// Frame Rate (E8h)
    ///
    /// ## Parameters
    ///
    /// * DINV `.0` => [`DINVMode`]
    /// * (unused) RTN1 `.1` => TODO
    /// * (undocumented) RTN2 `.2` => TODO (Misleading information)
    ///
    FrameRate(DINVMode),

    /// SPI 2data Control (E9h)
    ///
    /// ## Parameters
    ///
    /// * 2data_en `.0` => [`Data2EN`]
    /// * 2data_mdt `.1` => [`DataFormatMDT`]
    ///
    /// ## Restriction
    ///
    /// Inter command should be set high to enable this command
    ///
    Spi2dataControl(Data2EN, DataFormatMDT),

    /// Charge Pump Frequent Control (ECh)
    ///
    /// ## Parameters
    ///
    /// * avdd_clk_ad `.0` => TODO / Undocumented
    /// * avee_clk_ad `.1` => TODO / Undocumented
    /// * vcl_clk_ad `.2` => TODO / Undocumented
    /// * vgh_clk_ad `.3` => TODO / Undocumented
    /// * vgl_clk_ad `.4` => TODO / Undocumented
    ///
    ChargePumpFrequentControl(u8, u8, u8, u8, u8),

    /// Inner Register Enable 1 (FEh)
    ///
    /// ## Description
    ///
    /// This command is used for Inter_command controlling.
    /// To set Inter_command high ,you should write Inter register enable 1 (FEh) and Inter register
    /// enable 2 (EFh) continuously.
    /// Once Inter_command is set high, only hardware or software reset can turn it to low.
    ///
    InnerRegisterEnable1,

    /// Inner Register Enable 2 (EFh)
    ///
    /// ## Description
    ///
    /// This command is used for Inter_command controlling.
    /// To set Inter_command high ,you should write Inter register enable 1 (FEh) and Inter register
    /// enable 2 (EFh) continuously.
    /// Once Inter_command is set high, only hardware or software reset can turn it to low.
    ///
    InnerRegisterEnable2,

    /// Set GAMMA 1 (F0h)
    ///
    /// ## Parameters
    ///
    /// * Gamma1 `.0`
    ///
    /// ## Restriction
    ///
    /// Inter_command should be set high to enable this command
    ///
    SetGamma1(Gamma1),

    /// Set GAMMA 2 (F1h)
    ///
    /// ## Parameters
    ///
    /// * Gamma2 `.0`
    ///
    /// ## Restriction
    ///
    /// Inter_command should be set high to enable this command
    ///
    SetGamma2(Gamma2),

    /// Set GAMMA 3 (F2h)
    ///
    /// ## Parameters
    ///
    /// * Gamma3 `.0`
    ///
    /// ## Restriction
    ///
    /// Inter_command should be set high to enable this command
    ///
    SetGamma3(Gamma3),

    /// Set GAMMA 4 (F3h)
    ///
    /// ## Parameters
    ///
    /// * Gamma4 `.0`
    ///
    /// ## Restriction
    ///
    /// Inter_command should be set high to enable this command
    ///
    SetGamma4(Gamma4),

    /// Set Undocumented EBh (EBh)
    ///
    SetUndocumented0EBh(u8),

    /// Set Undocumented 084h (084h)
    ///
    SetUndocumented084h(u8),

    /// Set Undocumented 085h (085h)
    ///
    SetUndocumented085h(u8),

    /// Set Undocumented 086h (086h)
    ///
    SetUndocumented086h(u8),

    /// Set Undocumented 087h (087h)
    ///
    SetUndocumented087h(u8),

    /// Set Undocumented 088h (088h)
    ///
    SetUndocumented088h(u8),

    /// Set Undocumented 089h (089h)
    ///
    SetUndocumented089h(u8),

    /// Set Undocumented 08Ah (08Ah)
    ///
    SetUndocumented08Ah(u8),

    /// Set Undocumented 08Bh (08Bh)
    ///
    SetUndocumented08Bh(u8),

    /// Set Undocumented 08Ch (08Ch)
    ///
    SetUndocumented08Ch(u8),

    /// Set Undocumented 08Dh (08Dh)
    ///
    SetUndocumented08Dh(u8),

    /// Set Undocumented 08Eh (08Eh)
    ///
    SetUndocumented08Eh(u8),

    /// Set Undocumented 08Fh (08Fh)
    ///
    SetUndocumented08Fh(u8),

    /// Set Undocumented 090h (090h)
    ///
    SetUndocumented090h,

    /// Set Undocumented 062h (0x62h)
    ///
    SetUndocumented062h,

    /// Set Undocumented 063h (0x63h)
    ///
    SetUndocumented063h,

    /// Set Undocumented 064h (0x64h)
    ///
    SetUndocumented064h,

    /// Set Undocumented 066h (0x66h)
    ///
    SetUndocumented066h,

    /// Set Undocumented 067h (0x67h)
    ///
    SetUndocumented067h,

    /// Set Undocumented 074h (0x74h)
    ///
    SetUndocumented074h,

    /// Set Undocumented 098h (0x98h)
    ///
    SetUndocumented098h,

    ///Set SUndocumented 0BEh (0xBEh)
    SetUndocumented0BEh,
    ///Set SUndocumented 0BCh (0xBCh)
    SetUndocumented0BCh,
    ///Set SUndocumented 0BDh (0xBDh)
    SetUndocumented0BDh,
    ///Set SUndocumented 0E1h (0xE1h)
    SetUndocumented0E1h,
    ///Set SUndocumented 0DFh (0xDFh)
    SetUndocumented0DFh,
    ///Set SUndocumented 0EDh (0xEDh)
    SetUndocumented0EDh,
    ///Set SUndocumented 0AEh (0xAEh)
    SetUndocumented0AEh,
    ///Set SUndocumented 0CDh (0xCDh)
    SetUndocumented0CDh,
    ///Set SUndocumented 070h (0x70h)
    SetUndocumented070h,
    ///Set SUndocumented 0FFh (0xFFh)
    SetUndocumented0FFh,

    /// Memory Write (F2Ch)
    ///
    /// ## Description
    ///
    /// change to the other driver
    /// status. When this command is accepted, the column register and the page register are reset to
    /// the Start Column/Start
    /// Page positions. The Start Column/Start Page positions are different in accordance with
    /// MADCTL setting.) Then D [17:0] isstored in frame memory and the column register and the
    /// page register incremented. Sending any other command can stop frame Write. X = Don’t care
    ///
    /// ## Restriction
    ///
    /// In all color modes, there is no restriction on length of parameters.
    ///
    ///
    MemoryWrite,

    /// Write Memory Contiue (3Ch)
    ///
    /// ## Description
    ///
    /// This command transfers image data from the host processor to the display module’s frame
    /// memory continuing from the pixel location following the previous write_memory_continue or write_memory_start
    /// command.
    ///
    /// ### If set_address_mode B5 = 0:
    /// Data is written continuing from the pixel location after the write range of the previous
    /// write_memory_start or write_memory_continue. The column register is then incremented and pixels are written to the
    /// frame memory until the column register equals the End Column (EC) value. The column register is then reset to SC and the page register is
    /// incremented. Pixels are written to the frame memory until the page register equals the End Page
    /// (EP) value and the column register equals the EC value, or the host processor sends another command. If the
    /// number of pixels exceeds (EC –SC + 1) * (EP – SP + 1) the extra pixels are ignored.
    ///
    /// ### If set_address_mode B5 = 1:
    /// Data is written continuing from the pixel location after the write range of the previous
    /// write_memory_start or write_memory_continue. The page register is then incremented and pixels are written to the
    /// frame memory until the page register equals the End Page (EP) value. The page register is then
    /// reset to SP and the column register is incremented. Pixels are written to the frame memory until
    /// the column register equals the End column (EC) value and the page register equals the EP value,
    /// or the host processor sends another command. If the number of pixels exceeds (EC – SC + 1) *
    /// (EP –SP + 1) the extra pixels are ignored.
    /// Sending any other command can stop frame Write.
    /// Frame Memory Access and Interface setting (B3h), WEMODE=0
    /// When the transfer number of data exceeds (EC-SC+1)*(EP-SP+1), the exceeding data will be ignored.
    /// Frame Memory Access and Interface setting (B3h), WEMODE=1
    ///
    /// When the transfer number of data exceeds (EC-SC+1)*(EP-SP+1), the column and page number
    /// will be reset, and the
    /// exceeding data will be written into the following column and page
    ///
    /// ## Restriction
    ///
    /// A write_memory_start should follow a set_column_address, set_page_address or
    /// set_address_mode to define the write
    /// address. Otherwise, data written with write_memory_continue is written to undefined addresses
    MemoryWriteContinue,
}

impl Command {
    /// Send command to SSD1306
    pub fn send<DI>(self, iface: &mut DI) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
    {
        // 16bits command (2bytes)
        // 16bits param_1 (2bytes)
        // 16bits param_2 (2bytes)
        // 16bits param_3 (2bytes)
        // 16bits param_4 (2bytes)
        // Maximum 10 bytes
        // Array Size 5
        // Transform everything in 10 bytes array
        let (data, len): ([u8; 13], usize) = match self {
            Command::SleepMode(level) => (
                [
                    match level {
                        Logical::Off => 0x11,
                        Logical::On => 0x10,
                    },
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                1,
            ),
            Command::PartialMode => ([0x12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::NormalDisplayMode => ([0x13, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::DisplayInversion(level) => {
                ([0x20 | level as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1)
            }
            Command::DisplayState(level) => {
                ([0x28 | level as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1)
            }
            Command::ColumnAddressSet(sc, ec) => (
                [
                    0x2A,
                    (sc >> 8) as u8,
                    (sc & 0xFF) as u8,
                    (ec >> 8) as u8,
                    (ec & 0xFF) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                5,
            ),
            Command::RowAddressSet(sp, ep) => (
                [
                    0x2B,
                    (sp >> 8) as u8,
                    (sp & 0xFF) as u8,
                    (ep >> 8) as u8,
                    (ep & 0xFF) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                5,
            ),
            Command::VertialScrollDef(tfa, vsa) => (
                [
                    0x33,
                    (tfa >> 8) as u8,
                    (tfa & 0xFF) as u8,
                    (vsa >> 8) as u8,
                    (vsa & 0xFF) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                5,
            ),
            Command::TearingEffectLine(mode) => {
                ([0x34 | mode as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1)
            }
            Command::VerticalScrollStartAddresss(vsp) => (
                [
                    0x37,
                    (vsp >> 8) as u8,
                    (vsp & 0xFF) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                3,
            ),
            Command::IdleMode(mode) => ([0x38 | mode as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::PixelFormatSet(dbi, dpi) => (
                [
                    0x3A,
                    ((dpi as u8) << 4) | (dbi as u8),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::SetTearScanline(sts) => (
                [
                    0x44,
                    (((sts + 8) & 0x100) >> 8) as u8,
                    ((sts + 8) & 0xFF) as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                3,
            ),
            Command::DisplayBrightness(dbv) => ([0x51, dbv, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::CtrlDisplay(bctrl, dd, bl) => (
                [
                    0x53,
                    (bctrl as u8) << 5 | (dd as u8) << 3 | (bl as u8) << 2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::RGBInterfaceSignalCtrl(epl, dpl, hsp, vsp, rcm) => (
                [
                    0xB0,
                    (epl as u8 & 0b1)
                        | ((dpl as u8 & 0b1) << 1)
                        | ((hsp as u8 & 0b1) << 2)
                        | ((vsp as u8 & 0b1) << 3)
                        | ((rcm as u8 & 0b11) << 5),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::BlankingPorchControl(vfp, vbp, hbp) => (
                [
                    0xB5,
                    vfp,
                    vbp & 0b0111_1111,
                    hbp & 0b0001_1111,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                4,
            ),
            Command::DispalyFunctionControl(gs, ss, sm, nl) => (
                [
                    0xB6,
                    ((gs as u8 & 0b1) << 6) | ((ss as u8 & 0b1) << 5) | ((sm & 0b1) << 4),
                    nl & 0b0001_1111,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                3,
            ),
            Command::TEControl(te_pol, te_width) => (
                [
                    0xBA,
                    ((te_pol as u8) << 7) | (te_width & 0b0111_1111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::Interface(dm, rm, rim) => (
                [
                    0xF6,
                    ((dm as u8 & 0b11) << 2) | ((rm as u8 & 0b1) << 1) | (rim as u8 & 0b1),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::PowerCriterioControl(vcire) => (
                [
                    0xC1,
                    ((vcire as u8 & 0b1) << 1),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::VCoreVoltageControl(vddad) => (
                [
                    0xA7,
                    0b0100_0000 | vddad as u8,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::Vreg1aVoltageControl(value) => {
                ([0xC3, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::Vreg1bVoltageControl(value) => {
                ([0xC4, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::Vreg2aVoltageControl(value) => {
                ([0xC9, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::FrameRate(divn_mode) => (
                [
                    0xE8,
                    (divn_mode as u8 & 0b111) << 4,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::Spi2dataControl(data2_en, data_format) => (
                [
                    0xE9,
                    (data2_en as u8 & 0b1) << 3 | (data_format as u8 & 0b111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                3,
            ),
            Command::ChargePumpFrequentControl(
                avdd_clk_ad,
                avee_clk_ad,
                vcl_clk_ad,
                vgh_clk_ad,
                vgl_clk_ad,
            ) => (
                [
                    0xEC,
                    ((avdd_clk_ad & 0b111) << 4) | avee_clk_ad & 0b111,
                    vcl_clk_ad & 0b111,
                    ((vgh_clk_ad & 0b1111) << 4) | vgl_clk_ad & 0b1111,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                4,
            ),
            Command::InnerRegisterEnable1 => ([0xFE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::InnerRegisterEnable2 => ([0xEF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::SetGamma1(gamma) => (
                [
                    0xF0,
                    // 0b001, 0b000_101
                    (gamma.dig2j0_n & 0b11) << 6 | (gamma.vr1_n & 0b0011_1111),
                    (gamma.dig2j1_n & 0b11) << 6 | (gamma.vr2_n & 0b0011_1111),
                    (gamma.vr4_n & 0b0001_1111),
                    (gamma.vr6_n & 0b0001_1111),
                    (gamma.vr0_n & 0b1111) << 4 | (gamma.vr13_n & 0b0000_1111),
                    (gamma.vr20_n & 0b0111_1111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                7,
            ),
            Command::SetGamma2(gamma) => (
                [
                    0xF1,
                    (gamma.vr43_n & 0b0111_1111),
                    (gamma.vr27_n & 0b111) << 5 | (gamma.vr57_n & 0b0001_1111),
                    (gamma.vr36_n & 0b111) << 5 | (gamma.vr59_n & 0b0001_1111),
                    (gamma.vr61_n & 0b0011_1111),
                    (gamma.vr62_n & 0b0011_1111),
                    (gamma.vr50_n & 0b1111) << 4 | (gamma.vr63_n & 0b0000_1111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                7,
            ),
            Command::SetGamma3(gamma) => (
                [
                    0xF2,
                    (gamma.dig2j0_p & 0b11) << 6 | (gamma.vr1_p & 0b0011_1111),
                    (gamma.dig2j1_p & 0b11) << 6 | (gamma.vr2_p & 0b0011_1111),
                    (gamma.vr4_p & 0b0001_1111),
                    (gamma.vr6_p & 0b0001_1111),
                    (gamma.vr0_p & 0b1111) << 4 | (gamma.vr13_p & 0b0000_1111),
                    (gamma.vr20_p & 0b0111_1111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                7,
            ),
            Command::SetGamma4(gamma) => (
                [
                    0xF3,
                    (gamma.vr43_p & 0b0111_1111),
                    (gamma.vr27_p & 0b111) << 5 | (gamma.vr57_p & 0b0001_1111),
                    (gamma.vr36_p & 0b111) << 5 | (gamma.vr59_p & 0b0001_1111),
                    (gamma.vr61_p & 0b0011_1111),
                    (gamma.vr62_p & 0b0011_1111),
                    (gamma.vr50_p & 0b1111) << 4 | (gamma.vr63_p & 0b0000_1111),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                7,
            ),
            Command::MemoryWrite => ([0x2c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::MemoryWriteContinue => ([0x3c, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 1),
            Command::SetUndocumented0BEh => ([0xBE, 0x11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::SetUndocumented0BCh => ([0xBC, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::SetUndocumented0BDh => ([0xBD, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::SetUndocumented0E1h => ([0xE1, 0x10, 0x0E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 3),
            Command::SetUndocumented0DFh => {
                ([0xDF, 0x21, 0x0c, 0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4)
            }
            Command::SetUndocumented0EDh => ([0xED, 0x1B, 0x0B, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 3),
            Command::SetUndocumented0AEh => ([0xAE, 0x77, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::SetUndocumented0CDh => ([0xCD, 0x63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2),
            Command::SetUndocumented070h => (
                [
                    0x70, 0x07, 0x07, 0x04, 0x0E, 0x0F, 0x09, 0x07, 0x08, 0x03, 0, 0, 0,
                ],
                10,
            ),

            Command::SetUndocumented0FFh => {
                ([0xFF, 0x60, 0x01, 0x04, 0, 0, 0, 0, 0, 0, 0, 0, 0], 4)
            }
            Command::SetUndocumented0EBh(value) => {
                ([0xEB, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented084h(value) => {
                ([0x84, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented085h(value) => {
                ([0x85, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented086h(value) => {
                ([0x86, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented087h(value) => {
                ([0x87, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented088h(value) => {
                ([0x88, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented089h(value) => {
                ([0x89, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Ah(value) => {
                ([0x8A, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Bh(value) => {
                ([0x8B, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Ch(value) => {
                ([0x8C, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Dh(value) => {
                ([0x8D, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Eh(value) => {
                ([0x8E, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented08Fh(value) => {
                ([0x8F, value, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 2)
            }
            Command::SetUndocumented090h => {
                ([0x90, 0x08, 0x08, 0x08, 0x08, 0, 0, 0, 0, 0, 0, 0, 0], 5)
            }
            Command::MemoryAccessControl(my, mx, mv, ml, bgr, mh) => (
                [
                    0x36,
                    (my as u8) << 7
                        | (mx as u8) << 6
                        | (mv as u8) << 5
                        | (ml as u8) << 4
                        | (bgr as u8) << 3
                        | (mh as u8) << 2,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ],
                2,
            ),
            Command::SetUndocumented062h => (
                [
                    0x62, 0x18, 0x0D, 0x71, 0xED, 0x70, 0x70, 0x18, 0x0F, 0x71, 0xEF, 0x70, 0x70,
                ],
                13,
            ),
            Command::SetUndocumented063h => (
                [
                    0x63, 0x18, 0x11, 0x71, 0xF1, 0x70, 0x70, 0x18, 0x13, 0x71, 0xF3, 0x70, 0x70,
                ],
                13,
            ),
            Command::SetUndocumented064h => (
                [
                    0x64, 0x28, 0x29, 0xF1, 0x01, 0xF1, 0x00, 0x07, 0, 0, 0, 0, 0,
                ],
                8,
            ),
            Command::SetUndocumented066h => (
                [
                    0x66, 0x3C, 0x00, 0xCD, 0x67, 0x45, 0x45, 0x10, 0x00, 0x00, 0x00, 0, 0,
                ],
                11,
            ),
            Command::SetUndocumented067h => (
                [
                    0x67, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x01, 0x54, 0x10, 0x32, 0x98, 0, 0,
                ],
                11,
            ),
            Command::SetUndocumented074h => (
                [
                    0x74, 0x10, 0x85, 0x80, 0x00, 0x00, 0x4E, 0x00, 0, 0, 0, 0, 0,
                ],
                8,
            ),
            Command::SetUndocumented098h => ([0x98, 0x3e, 0x07, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 3),
        };

        // Send command over the interface
        // TODO: do something better
        iface.send_commands(U8(&[data[0]]))?;
        if len > 1 {
            iface.send_data(U8(&data[1..len]))?;
        }
        Ok(())
    }
}

/// Logical On/Off
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Logical {
    Off = 0,
    On = 1,
}

impl From<bool> for Logical {
    fn from(val: bool) -> Logical {
        match val {
            true => Logical::On,
            false => Logical::Off,
        }
    }
}

impl From<u8> for Logical {
    fn from(val: u8) -> Logical {
        match val {
            0 => Logical::Off,
            _ => Logical::On,
        }
    }
}

/// Display Enable Polarity (DE Polarity)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DEPolarity {
    /// High enable for RGB interface
    HighEnableForRGB = 0,
    /// Low enable for RGB interface
    LowEnableForRGB = 1,
}

impl From<bool> for DEPolarity {
    fn from(val: bool) -> DEPolarity {
        match val {
            true => DEPolarity::HighEnableForRGB,
            false => DEPolarity::LowEnableForRGB,
        }
    }
}

impl From<u8> for DEPolarity {
    fn from(val: u8) -> DEPolarity {
        match val {
            0 => DEPolarity::HighEnableForRGB,
            _ => DEPolarity::LowEnableForRGB,
        }
    }
}

/// The Tearing Effect output signal pulse polarity
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum TEPolarity {
    /// High enable for RGB interface
    PositivePulse = 0,
    /// Low enable for RGB interface
    NegativePulse = 1,
}

impl From<bool> for TEPolarity {
    fn from(val: bool) -> TEPolarity {
        match val {
            true => TEPolarity::PositivePulse,
            false => TEPolarity::NegativePulse,
        }
    }
}

impl From<u8> for TEPolarity {
    fn from(val: u8) -> TEPolarity {
        match val {
            0 => TEPolarity::PositivePulse,
            _ => TEPolarity::NegativePulse,
        }
    }
}

/// Display Enable Polarity (DOTCLK Polarity)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DOTClk {
    /// Data fetched at the rising time
    FetchOnRising = 0,
    /// Data fetched at the falling time
    FetchOnFalling = 1,
}

impl From<bool> for DOTClk {
    fn from(val: bool) -> DOTClk {
        match val {
            true => DOTClk::FetchOnRising,
            false => DOTClk::FetchOnFalling,
        }
    }
}

impl From<u8> for DOTClk {
    fn from(val: u8) -> DOTClk {
        match val {
            0 => DOTClk::FetchOnRising,
            _ => DOTClk::FetchOnFalling,
        }
    }
}

/// Polarity Clock Sync
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum XSpl {
    /// Low level sync clock
    LowSyncClock = 0,
    /// Low level sync clock
    HighSyncClock = 1,
}

impl From<bool> for XSpl {
    fn from(val: bool) -> XSpl {
        match val {
            true => XSpl::LowSyncClock,
            false => XSpl::HighSyncClock,
        }
    }
}

impl From<u8> for XSpl {
    fn from(val: u8) -> XSpl {
        match val {
            0 => XSpl::LowSyncClock,
            _ => XSpl::HighSyncClock,
        }
    }
}

/// Polarity Clock Sync
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RCMMode {
    /// DE Mode Valid data is determined by the DE signal
    DEMode = 0b10,
    /// SYNC Mode In SYNC mode, DE signal is ignored; blanking porch
    /// is determined by B5h command
    SyncMode = 0b11,
}

impl From<u8> for RCMMode {
    fn from(val: u8) -> RCMMode {
        match val {
            0b10 => RCMMode::DEMode,
            _ => RCMMode::SyncMode,
        }
    }
}

/// Output Scan Direction
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum SSMode {
    /// To assign R, G, B dots to the source driver pins from S1 to S360, set SS = 0
    S1toS360 = 0,
    /// To assign R, G, B dots to the source driver pins from S360 to S1, set SS = 1.
    S360toS1 = 1,
}

impl From<u8> for SSMode {
    fn from(val: u8) -> SSMode {
        match val {
            0 => SSMode::S1toS360,
            _ => SSMode::S1toS360,
        }
    }
}

/// Display Operation Mode
/// Select the display operation mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DMMode {
    /// Internal clock operation
    InternalClockOperation = 0,
    /// RGB Interface Mode
    RGBInterfaceMode = 1,
    /// VSYNC Interface Mode
    VSYNCInterfaceMode = 2,
    /// RGB Interface Mode
    SettingDisabled = 3,
}

impl From<u8> for DMMode {
    fn from(val: u8) -> DMMode {
        match val {
            0 => DMMode::InternalClockOperation,
            1 => DMMode::RGBInterfaceMode,
            2 => DMMode::VSYNCInterfaceMode,
            _ => DMMode::SettingDisabled,
        }
    }
}

/// Interface for RAM Access
/// Select the interface to access the GRAM.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RMMode {
    /// Select System or VSync Interface to write in GRAM
    SystemOrVSyncInterface = 0,
    /// Select RGB Interface to write in GRAM
    RGBInterface = 1,
}

impl From<u8> for RMMode {
    fn from(val: u8) -> RMMode {
        match val {
            0 => RMMode::SystemOrVSyncInterface,
            1 => RMMode::RGBInterface,
            _ => RMMode::RGBInterface,
        }
    }
}

/// RGB Interface Mode
/// Specify the RGB interface mode when the RGB interface is used.
/// These bit should be set before display operation through the RGB interface
/// and should not be set during operation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum RIMMode {
    /// 18- bit RGB interface (1 transfer/pixel)
    /// 16- bit RGB interface (1 transfer/pixel)
    TransferPerPixel1 = 0,
    /// 6- bit RGB interface (3 transfer/pixel)
    TransferPerPixel3 = 1,
}

impl From<u8> for RIMMode {
    fn from(val: u8) -> RIMMode {
        match val {
            0 => RIMMode::TransferPerPixel1,
            1 => RIMMode::TransferPerPixel3,
            _ => RIMMode::TransferPerPixel3,
        }
    }
}

/// Display Inversion Mode
/// Set display inversion mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DINVMode {
    /// column inversion
    ColumnInversion = 0,
    /// 1 dot inversion
    Inversion1Dot = 1,
    /// 2 dot inversion
    Inversion2Dot = 2,
    /// 4 dot inversion
    Inversion4Dot = 3,
    /// 8 dot inversion
    Inversion8Dot = 4,
}

impl From<u8> for DINVMode {
    fn from(val: u8) -> DINVMode {
        match val {
            0 => DINVMode::ColumnInversion,
            1 => DINVMode::Inversion1Dot,
            2 => DINVMode::Inversion2Dot,
            3 => DINVMode::Inversion4Dot,
            4 => DINVMode::Inversion8Dot,
            _ => DINVMode::Inversion8Dot,
        }
    }
}

/// 2 Data Line Mode 3/4-wire SPI
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Data2EN {
    /// 3-wire SPI
    Data3Wire = 0,
    /// 4-wire SPI
    Data4Wire = 1,
}

impl From<u8> for Data2EN {
    fn from(val: u8) -> Data2EN {
        match val {
            0 => Data2EN::Data3Wire,
            1 => Data2EN::Data4Wire,
            _ => Data2EN::Data4Wire,
        }
    }
}

/// DataFormat MDT
/// Set Pixel Data Format in 2_data_line mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum DataFormatMDT {
    /// 65K color 1pixle/transition
    Color65k1PixelPerTransition = 0,
    /// 262K color 1pixle/transition
    Color262k1PixelPerTransition = 1,
    /// 262K color 2/3pixle/transition
    Color262k2Or3PixelPerTransition = 2,
    /// 4M color 1pixle/transition
    Color4Mk1PixelPerTransition = 4,
    /// 4M color 2/3pixle/transition
    Color4M2Or3PixelPerTransition = 5,
}

impl From<u8> for DataFormatMDT {
    fn from(val: u8) -> DataFormatMDT {
        match val {
            0 => DataFormatMDT::Color65k1PixelPerTransition,
            1 => DataFormatMDT::Color262k1PixelPerTransition,
            2 => DataFormatMDT::Color262k2Or3PixelPerTransition,
            3 => DataFormatMDT::Color4Mk1PixelPerTransition,
            4 => DataFormatMDT::Color4M2Or3PixelPerTransition,
            _ => DataFormatMDT::Color4M2Or3PixelPerTransition,
        }
    }
}

/// External reference voltage Vci or internal reference voltage VCIT
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VCIRe {
    /// Internal reference voltage 2.5V (default)
    Internal = 0,
    /// External reference voltage Vci
    External = 1,
}

impl From<u8> for VCIRe {
    fn from(val: u8) -> VCIRe {
        match val {
            0 => VCIRe::Internal,
            1 => VCIRe::External,
            _ => VCIRe::External,
        }
    }
}

/// Voltage level value to output the VCORE level,
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum VddAd {
    VCore1_483V = 0x00,
    VCore1_545V = 0x01,
    VCore1_590V = 0x02,
    VCore1_638V = 0x03,
    VCore1_714V = 0x04,
    VCore1_279V = 0x05,
    VCore1_859V = 0x06,
    VCore1_925V = 0x07,
    VCore1_994V = 0x08,
    VCore2_109V = 0x09,
    VCore2_193V = 0x0a,
    VCore2_286V = 0x0b,
    VCore2_385V = 0x0c,
    VCore1_713V = 0x0d,
    VCore1_713Ve = 0x0e,
    VCore1_713Vf = 0x0f,
}

impl From<u8> for VddAd {
    fn from(val: u8) -> VddAd {
        match val {
            0x00 => VddAd::VCore1_483V,
            0x01 => VddAd::VCore1_545V,
            0x02 => VddAd::VCore1_590V,
            0x03 => VddAd::VCore1_638V,
            0x04 => VddAd::VCore1_714V,
            0x05 => VddAd::VCore1_279V,
            0x06 => VddAd::VCore1_859V,
            0x07 => VddAd::VCore1_925V,
            0x08 => VddAd::VCore1_994V,
            0x09 => VddAd::VCore2_109V,
            0x0a => VddAd::VCore2_193V,
            0x0b => VddAd::VCore2_286V,
            0x0c => VddAd::VCore2_385V,
            0x0d => VddAd::VCore1_713V,
            0x0e => VddAd::VCore1_713Ve,
            0x0f => VddAd::VCore1_713Vf,
            _ => VddAd::VCore1_713Vf,
        }
    }
}

/// Gate Output Scan Direction
/// Sets the direction of scan by the gate driver in the range determined by SCN [4:0] and NL
/// [4:0]. The scan direction determined by GS = 0 can be reversed by setting GS = 1.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum GSMode {
    G1toG32 = 0,
    G32toG1 = 1,
}

impl From<u8> for GSMode {
    fn from(val: u8) -> GSMode {
        match val {
            0 => GSMode::G1toG32,
            _ => GSMode::G32toG1,
        }
    }
}

/// Dpi is the pixel format select of RGB interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Dpi {
    Pixel16bits = 0b0000_0101,
    Pixel18bits = 0b0000_0110,
}

/// Dbi is the pixel format of MCU interface.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Dbi {
    Pixel12bits = 0b0000_0011,
    Pixel16bits = 0b0000_0101,
    Pixel18bits = 0b0000_0110,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Gamma1 {
    /// dig2gam_dig2j0_n
    pub dig2j0_n: u8,
    /// dig2gam_vr1_n
    pub vr1_n: u8,
    /// dig2gam_dig2j1_n
    pub dig2j1_n: u8,
    /// dig2gam_vr2_n
    pub vr2_n: u8,
    /// dig2gam_vr4_n
    pub vr4_n: u8,
    /// dig2gam_vr6_n
    pub vr6_n: u8,
    /// dig2gam_vr0_n
    pub vr0_n: u8,
    /// dig2gam_vr13_n
    pub vr13_n: u8,
    /// dig2gam_vr20_n
    pub vr20_n: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Gamma2 {
    /// dig2gam_vr43_n
    pub vr43_n: u8,
    /// dig2gam_vr27_n
    pub vr27_n: u8,
    /// dig2gam_vr57_n
    pub vr57_n: u8,
    /// dig2gam_vr36_n
    pub vr36_n: u8,
    /// dig2gam_vr59_n
    pub vr59_n: u8,
    /// dig2gam_vr61_n
    pub vr61_n: u8,
    /// dig2gam_vr62_n
    pub vr62_n: u8,
    /// dig2gam_vr50_n
    pub vr50_n: u8,
    /// dig2gam_vr63_n
    pub vr63_n: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Gamma3 {
    /// dig2gam_dig2j0_p
    pub dig2j0_p: u8,
    /// dig2gam_vr1_p
    pub vr1_p: u8,
    /// dig2gam_dig2j1_p
    pub dig2j1_p: u8,
    /// dig2gam_vr2_p
    pub vr2_p: u8,
    /// dig2gam_vr4_p
    pub vr4_p: u8,
    /// dig2gam_vr6_p
    pub vr6_p: u8,
    /// dig2gam_vr0_p
    pub vr0_p: u8,
    /// dig2gam_vr13_p
    pub vr13_p: u8,
    /// dig2gam_vr20_p
    pub vr20_p: u8,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Gamma4 {
    /// dig2gam_vr43_p
    pub vr43_p: u8,
    /// dig2gam_vr27_p
    pub vr27_p: u8,
    /// dig2gam_vr57_p
    pub vr57_p: u8,
    /// dig2gam_vr36_p
    pub vr36_p: u8,
    /// dig2gam_vr59_p
    pub vr59_p: u8,
    /// dig2gam_vr61_p
    pub vr61_p: u8,
    /// dig2gam_vr62_p
    pub vr62_p: u8,
    /// dig2gam_vr50_p
    pub vr50_p: u8,
    /// dig2gam_vr63_p
    pub vr63_p: u8,
}
