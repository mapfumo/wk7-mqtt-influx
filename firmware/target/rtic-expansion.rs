#[doc = r" The RTIC application module"] pub mod app
{
    #[doc =
    r" Always include the device crate which contains the vector table"] use
    stm32f4xx_hal :: pac as
    you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml; use
    stm32f4xx_hal ::
    {
        gpio :: { Output, Pin }, i2c :: I2c, pac, prelude :: * , rcc ::
        Config, serial ::
        { Config as SerialConfig, Event as SerialEvent, Serial }, timer ::
        { CounterHz, Event },
    }; use core :: fmt :: Write as _; use display_interface_i2c ::
    I2CInterface; use embedded_graphics ::
    {
        mono_font :: { ascii :: FONT_6X10, MonoTextStyleBuilder }, pixelcolor
        :: BinaryColor, prelude :: * , text :: Text,
    }; use heapless :: { String, Vec }; use shared_bus :: CortexMMutex; use
    ssd1306 :: { mode :: BufferedGraphicsMode, prelude :: * , Ssd1306 }; use
    sht3x :: { SHT3x, Repeatability, Address as ShtAddress }; use serde ::
    { Deserialize, Serialize }; #[doc = r" User code from within the module"]
    const NODE_ID : & str = "N2"; const RX_BUFFER_SIZE : usize = 255; const
    NETWORK_ID : u8 = 18; const LORA_FREQ : u32 = 915;
    #[doc = " Sensor data packet for binary transmission (must match Node 1)"]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)] pub struct
    SensorDataPacket
    {
        pub seq_num : u16, pub temperature : i16, pub humidity : u16, pub
        gas_resistance : u32,
    } #[doc = " ACK/NACK packet for acknowledgment (must match Node 1)"]
    #[doc = " Size: 3 bytes (1 byte msg_type + 2 bytes seq_num)"]
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)] pub struct AckPacket
    { pub msg_type : u8, pub seq_num : u16, } const MSG_TYPE_ACK : u8 = 1;
    const MSG_TYPE_NACK : u8 = 2;
    #[doc = " Calculate CRC-16 checksum for data integrity"] fn
    calculate_crc16(data : & [u8]) -> u16
    {
        use crc :: { Crc, CRC_16_IBM_3740 }; const CRC16 : Crc < u16 > = Crc
        :: < u16 > :: new(& CRC_16_IBM_3740); CRC16.checksum(data)
    } #[doc = " Send ACK packet to Node 1"]
    #[doc = " Format: AT+SEND=1,<length>,<binary_ack_packet>\\r\\n"] fn
    send_ack(uart : & mut Serial < pac :: UART4 > , seq_num : u16, is_ack :
    bool)
    {
        use core :: fmt :: Write; use heapless :: String; let ack_packet =
        AckPacket
        {
            msg_type : if is_ack { MSG_TYPE_ACK } else { MSG_TYPE_NACK },
            seq_num,
        }; let mut ack_buffer = [0u8; 8]; match postcard ::
        to_slice(& ack_packet, & mut ack_buffer)
        {
            Ok(serialized) =>
            {
                let ack_len = serialized.len(); let cmd_prefix = "AT+SEND=1,";
                for b in cmd_prefix.as_bytes()
                { let _ = nb :: block! (uart.write(*b)); } let mut len_str :
                String < 8 > = String :: new(); let _ = core :: write!
                (len_str, "{},", ack_len); for b in len_str.as_bytes()
                { let _ = nb :: block! (uart.write(*b)); } for b in serialized
                { let _ = nb :: block! (uart.write(*b)); } let _ = nb ::
                block! (uart.write(b'\r')); let _ = nb :: block!
                (uart.write(b'\n')); defmt :: info!
                ("{} sent for packet #{}", if is_ack { "ACK" } else
                { "NACK" }, seq_num);
            } Err(_) =>
            { defmt :: error! ("Failed to serialize ACK packet"); }
        }
    } pub struct I2cCompat < I2C > (pub I2C); impl < I2C > embedded_hal_0_2 ::
    blocking :: i2c :: Write for I2cCompat < I2C > where I2C : embedded_hal ::
    i2c :: I2c,
    {
        type Error = I2C :: Error; fn
        write(& mut self, addr : u8, bytes : & [u8]) -> Result < (), Self ::
        Error > { self.0.write(addr, bytes) }
    } impl < I2C > embedded_hal_0_2 :: blocking :: i2c :: Read for I2cCompat <
    I2C > where I2C : embedded_hal :: i2c :: I2c,
    {
        type Error = I2C :: Error; fn
        read(& mut self, addr : u8, buffer : & mut [u8]) -> Result < (), Self
        :: Error > { self.0.read(addr, buffer) }
    } impl < I2C > embedded_hal_0_2 :: blocking :: i2c :: WriteRead for
    I2cCompat < I2C > where I2C : embedded_hal :: i2c :: I2c,
    {
        type Error = I2C :: Error; fn
        write_read(& mut self, addr : u8, bytes : & [u8], buffer : & mut
        [u8],) -> Result < (), Self :: Error >
        { self.0.write_read(addr, bytes, buffer) }
    } type MyI2c = I2c < pac :: I2C1 > ; type BusManager = shared_bus ::
    BusManager < CortexMMutex < I2cCompat < MyI2c > > > ; type I2cProxy =
    shared_bus :: I2cProxy < 'static, CortexMMutex < I2cCompat < MyI2c > > > ;
    type LoraDisplay = Ssd1306 < I2CInterface < I2cProxy > ,
    DisplaySize128x64, BufferedGraphicsMode < DisplaySize128x64 > > ;
    #[derive(Debug, Clone, Copy)] pub struct SensorData
    {
        pub temperature : f32, pub humidity : f32, pub gas_resistance : u32,
        pub packet_num : u16,
    } type ShtDelay = stm32f4xx_hal :: timer :: Delay < pac :: TIM5, 1000000 >
    ; #[derive(Debug, Clone, Copy)] pub struct ParsedMessage
    { pub sensor_data : SensorData, pub rssi : i16, pub snr : i16, } fn
    send_at_command(uart : & mut Serial < pac :: UART4 > , cmd : & str)
    {
        defmt :: info! ("Sending AT command: {}", cmd); for byte in
        cmd.as_bytes() { let _ = nb :: block! (uart.write(*byte)); } let _ =
        nb :: block! (uart.write(b'\r')); let _ = nb :: block!
        (uart.write(b'\n')); cortex_m :: asm :: delay(8_400_000);
    } #[doc = " Parse binary LoRa message from RYLR998"]
    #[doc =
    " Format: +RCV=<Address>,<Length>,<BinaryData>,<RSSI>,<SNR>\\r\\n"]
    #[doc = " where <BinaryData> is postcard-serialized SensorDataPacket"] fn
    parse_binary_lora_message(buffer : & [u8]) -> Option < ParsedMessage >
    {
        if buffer.len() < 10 || & buffer [0 .. 5] != b"+RCV=" { return None; }
        let mut comma1_pos = None; let mut comma2_pos = None; for (i, & byte)
        in buffer [5 ..].iter().enumerate()
        {
            if byte == b','
            {
                if comma1_pos.is_none() { comma1_pos = Some(5 + i); } else if
                comma2_pos.is_none() { comma2_pos = Some(5 + i); break; }
            }
        } let comma1 = comma1_pos ? ; let comma2 = comma2_pos ? ; let
        len_bytes = & buffer [comma1 + 1 .. comma2]; let len_str = core :: str
        :: from_utf8(len_bytes).ok() ? ; let payload_len : usize =
        len_str.parse().ok() ? ; let payload_start = comma2 + 1; let
        payload_end = payload_start + payload_len; if payload_end >
        buffer.len()
        { defmt :: warn! ("Payload exceeds buffer"); return None; } let
        binary_payload = & buffer [payload_start .. payload_end]; if
        binary_payload.len() < 3
        {
            defmt :: warn! ("Payload too short for CRC validation"); return
            None;
        } let data_len = binary_payload.len() - 2; let data_bytes = &
        binary_payload [0 .. data_len]; let received_crc_high = binary_payload
        [data_len]; let received_crc_low = binary_payload [data_len + 1]; let
        received_crc = ((received_crc_high as u16) << 8) |
        (received_crc_low as u16); let calculated_crc =
        calculate_crc16(data_bytes); if received_crc != calculated_crc
        {
            defmt :: error!
            ("CRC FAIL! Received: 0x{:04X}, Calculated: 0x{:04X}",
            received_crc, calculated_crc); return None;
        } defmt :: info! ("CRC OK: 0x{:04X}", received_crc); let sensor_packet
        : SensorDataPacket = match postcard :: from_bytes(data_bytes)
        {
            Ok(pkt) => pkt, Err(_) =>
            {
                defmt :: error! ("Postcard deserialization failed"); return
                None;
            }
        }; let after_payload_bytes = & buffer [payload_end ..]; let
        after_payload_str = core :: str :: from_utf8(after_payload_bytes).ok()
        ? ; let parts : Vec < & str, 4 > =
        after_payload_str.split(',').collect(); if parts.len() < 3
        { return None; } let rssi : i16 = parts [1].parse().ok() ? ; let snr :
        i16 = parts [2].trim().parse().ok() ? ; let temp_c =
        sensor_packet.temperature as f32 / 10.0; let humid_pct =
        sensor_packet.humidity as f32 / 100.0;
        Some(ParsedMessage
        {
            sensor_data : SensorData
            {
                temperature : temp_c, humidity : humid_pct, gas_resistance :
                sensor_packet.gas_resistance, packet_num :
                sensor_packet.seq_num,
            }, rssi, snr,
        })
    } #[doc = " Format telemetry as JSON for USB output"]
    #[doc = " Returns newline-delimited JSON (NDJSON format)"] fn
    format_json_telemetry(parsed : & ParsedMessage, timestamp_ms : u32,
    packets_received : u32, crc_errors : u32, gateway_temp : Option < f32 > ,
    gateway_humidity : Option < f32 > ,) -> heapless :: String < 512 >
    {
        use core :: fmt :: Write; let mut json = heapless :: String :: < 512 >
        :: new(); let _ = write! (json, "{{\"ts\":{},", timestamp_ms); let _ =
        write! (json, "\"id\":\"N2\","); let temp =
        parsed.sensor_data.temperature; let hum = parsed.sensor_data.humidity;
        let gas = parsed.sensor_data.gas_resistance; let _ = write!
        (json, "\"n1\":{{"); let _ = write! (json, "\"t\":{:.1},", temp); let
        _ = write! (json, "\"h\":{:.1},", hum); let _ = write!
        (json, "\"g\":{}", gas); let _ = write! (json, "}},"); let _ = write!
        (json, "\"n2\":{{"); if let Some(t) = gateway_temp
        {
            let _ = write! (json, "\"t\":{:.1}", t); if
            gateway_humidity.is_some() { let _ = write! (json, ","); }
        } if let Some(h) = gateway_humidity
        { let _ = write! (json, "\"h\":{:.1}", h); } let _ = write!
        (json, "}},"); let _ = write! (json, "\"sig\":{{"); let _ = write!
        (json, "\"rssi\":{},", parsed.rssi); let _ = write!
        (json, "\"snr\":{}", parsed.snr); let _ = write! (json, "}},"); let _
        = write! (json, "\"sts\":{{"); let _ = write!
        (json, "\"rx\":{},", packets_received); let _ = write!
        (json, "\"err\":{}", crc_errors); let _ = write! (json, "}}}}\\n");
        json
    } #[doc = r" User code end"] #[doc = " User provided init function"]
    #[inline(always)] #[allow(non_snake_case)] fn init(cx : init :: Context)
    -> (Shared, Local, init :: Monotonics)
    {
        let dp = cx.device; let mut rcc =
        dp.RCC.freeze(Config :: hsi().sysclk(84.MHz())); let gpioa =
        dp.GPIOA.split(& mut rcc); let gpiob = dp.GPIOB.split(& mut rcc); let
        gpioc = dp.GPIOC.split(& mut rcc); let led =
        gpioa.pa5.into_push_pull_output(); let tx =
        gpioc.pc10.into_alternate(); let rx = gpioc.pc11.into_alternate(); let
        mut lora_uart = Serial ::
        new(dp.UART4, (tx, rx), SerialConfig ::
        default().baudrate(115200.bps()), & mut rcc,).unwrap(); defmt :: info!
        ("Configuring LoRa module (Node 2)...");
        send_at_command(& mut lora_uart, "AT");
        send_at_command(& mut lora_uart, "AT+ADDRESS=2"); let mut cmd_buf :
        String < 32 > = String :: new(); let _ = core :: write!
        (cmd_buf, "AT+NETWORKID={}", NETWORK_ID);
        send_at_command(& mut lora_uart, cmd_buf.as_str()); cmd_buf.clear();
        let _ = core :: write! (cmd_buf, "AT+BAND={}000000", LORA_FREQ);
        send_at_command(& mut lora_uart, cmd_buf.as_str());
        send_at_command(& mut lora_uart, "AT+PARAMETER=7,9,1,7"); while
        lora_uart.read().is_ok() {} let uart_ptr = unsafe
        { & * pac :: UART4 :: ptr() }; let sr = uart_ptr.sr().read(); if
        sr.ore().bit_is_set() || sr.nf().bit_is_set() || sr.fe().bit_is_set()
        {
            let _ = uart_ptr.dr().read(); defmt :: info!
            ("N2 INIT: Cleared error flags (ORE={} NF={} FE={})",
            sr.ore().bit_is_set(), sr.nf().bit_is_set(),
            sr.fe().bit_is_set());
        } defmt :: info! ("LoRa module configured");
        lora_uart.listen(SerialEvent :: RxNotEmpty); let scl =
        gpiob.pb8.into_alternate_open_drain(); let sda =
        gpiob.pb9.into_alternate_open_drain(); let i2c = I2c ::
        new(dp.I2C1, (scl, sda), 100.kHz(), & mut rcc); let i2c_compat =
        I2cCompat(i2c); let bus : & 'static BusManager = shared_bus ::
        new_cortexm! (I2cCompat<MyI2c> = i2c_compat).unwrap(); let interface =
        I2CInterface :: new(bus.acquire_i2c(), 0x3C, 0x40); let mut display =
        Ssd1306 ::
        new(interface, DisplaySize128x64, DisplayRotation ::
        Rotate0).into_buffered_graphics_mode(); display.init().unwrap(); let
        style = MonoTextStyleBuilder ::
        new().font(& FONT_6X10).text_color(BinaryColor :: On).build(); let _ =
        display.clear(BinaryColor :: Off); Text ::
        new("N2 RECEIVER", Point :: new(0, 8),
        style).draw(& mut display).ok(); let mut init_buf : String < 32 > =
        String :: new(); let _ = core :: write!
        (init_buf, "Net:{} {}MHz", NETWORK_ID, LORA_FREQ); Text ::
        new(& init_buf, Point :: new(0, 20), style).draw(& mut display).ok();
        Text ::
        new("Waiting...", Point :: new(0, 32),
        style).draw(& mut display).ok(); let _ = display.flush(); defmt ::
        info! ("Initializing USART2 (ST-Link VCP) for JSON output..."); let
        vcp_tx = gpioa.pa2.into_alternate(); let vcp_rx =
        gpioa.pa3.into_alternate(); let vcp_uart = Serial ::
        new(dp.USART2, (vcp_tx, vcp_rx), SerialConfig ::
        default().baudrate(115200.bps()), & mut rcc,).unwrap(); defmt :: info!
        ("USART2 VCP initialized at 115200 baud"); defmt :: info!
        ("Initializing SHT3x sensor..."); let sht_delay =
        dp.TIM5.delay_us(& mut rcc); let sht3x_sensor = SHT3x ::
        new(bus.acquire_i2c(), sht_delay, ShtAddress :: Low); defmt :: info!
        ("SHT3x initialized at 0x44"); let mut timer =
        dp.TIM2.counter_hz(& mut rcc); timer.start(2.Hz()).unwrap();
        timer.listen(Event :: Update);
        (Shared
        {
            lora_uart, vcp_uart, display, last_packet : None, packets_received
            : 0, crc_errors : 0, sht3x : Some(sht3x_sensor), sht3x_skip_reads
            : 4, gateway_temp : None, gateway_humidity : None, uptime_ms : 0,
        }, Local { led, timer, rx_buffer : Vec :: new(), }, init ::
        Monotonics(),)
    } #[doc = " User HW task: tim2_handler"] #[allow(non_snake_case)] fn
    tim2_handler(mut cx : tim2_handler :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ;
        cx.local.timer.clear_flags(stm32f4xx_hal :: timer :: Flag :: Update);
        cx.local.led.toggle();
        cx.shared.uptime_ms.lock(| uptime | * uptime += 500); let should_read
        =
        cx.shared.sht3x_skip_reads.lock(| skip |
        { if * skip > 0 { * skip -= 1; false } else { true } }); if
        should_read
        {
            cx.shared.sht3x.lock(| sht_opt |
            {
                if let Some(sht) = sht_opt
                {
                    if let Ok(measurement) = sht.measure(Repeatability :: High)
                    {
                        let temp = measurement.temperature as f32 / 100.0; let
                        humidity = measurement.humidity as f32 / 100.0;
                        cx.shared.gateway_temp.lock(| t | * t = Some(temp));
                        cx.shared.gateway_humidity.lock(| h | * h = Some(humidity));
                        defmt :: info!
                        ("SHT3x read: T={}°C, H={}%", temp, humidity);
                    }
                }
            });
        } let packet_copy = cx.shared.last_packet.lock(| pkt_opt | * pkt_opt);
        let total_count = cx.shared.packets_received.lock(| count | * count);
        defmt :: info!
        ("N2 Timer: total_count={}, has_packet={}", total_count,
        packet_copy.is_some()); if let Some(parsed) = packet_copy
        {
            cx.shared.display.lock(| disp |
            {
                let _ = disp.clear(BinaryColor :: Off); let style =
                MonoTextStyleBuilder ::
                new().font(& FONT_6X10).text_color(BinaryColor :: On).build();
                let mut buf : String < 64 > = String :: new(); let _ = core ::
                write!
                (buf, "T:{:.1}C H:{:.0}%", parsed.sensor_data.temperature,
                parsed.sensor_data.humidity); Text ::
                new(& buf, Point :: new(0, 8), style).draw(disp).ok();
                buf.clear(); let _ = core :: write!
                (buf, "Gas:{:.0}k", parsed.sensor_data.gas_resistance as f32 /
                1000.0); Text ::
                new(& buf, Point :: new(0, 20), style).draw(disp).ok();
                buf.clear(); let _ = core :: write!
                (buf, "{} RX #{:04}", NODE_ID, parsed.sensor_data.packet_num);
                Text ::
                new(& buf, Point :: new(0, 32), style).draw(disp).ok();
                buf.clear(); let _ = core :: write!
                (buf, "Net:{} {}MHz", NETWORK_ID, LORA_FREQ); Text ::
                new(& buf, Point :: new(0, 44), style).draw(disp).ok();
                buf.clear(); let _ = core :: write!
                (buf, "RSSI:{} SNR:{} #{}", parsed.rssi, parsed.snr,
                total_count); Text ::
                new(& buf, Point :: new(0, 56), style).draw(disp).ok(); let _
                = disp.flush();
            });
        }
    } #[doc = " User HW task: uart4_handler"] #[allow(non_snake_case)] fn
    uart4_handler(mut cx : uart4_handler :: Context)
    {
        use rtic :: Mutex as _; use rtic :: mutex :: prelude :: * ; let
        uart_ptr = unsafe { & * pac :: UART4 :: ptr() }; let sr =
        uart_ptr.sr().read(); let has_ore = sr.ore().bit_is_set(); let has_fe
        = sr.fe().bit_is_set(); let has_ne = sr.nf().bit_is_set(); if has_ore
        || has_fe || has_ne
        {
            let _ = uart_ptr.dr().read(); defmt :: warn!
            ("UART errors cleared: ORE={} FE={} NF={}", has_ore, has_fe,
            has_ne);
        } let mut should_process = false; let mut bytes_read = 0u16;
        cx.shared.lora_uart.lock(| uart |
        {
            while let Ok(byte) = uart.read()
            {
                bytes_read += 1; if cx.local.rx_buffer.len() < RX_BUFFER_SIZE
                { let _ = cx.local.rx_buffer.push(byte); } if byte == b'\n'
                { should_process = true; }
            }
        }); if bytes_read > 0
        {
            defmt :: info!
            ("UART INT: {} bytes, complete={}", bytes_read, should_process);
        } if should_process
        {
            defmt :: info!
            ("Processing buffer: {} bytes", cx.local.rx_buffer.len()); if let
            Ok(msg_text) = core :: str ::
            from_utf8(cx.local.rx_buffer.as_slice())
            { defmt :: info! ("Buffer as text: {}", msg_text); } if let
            Some(parsed) =
            parse_binary_lora_message(cx.local.rx_buffer.as_slice())
            {
                defmt :: info!
                ("Binary RX - T:{} H:{} G:{} Pkt:{} RSSI:{} SNR:{}",
                parsed.sensor_data.temperature, parsed.sensor_data.humidity,
                parsed.sensor_data.gas_resistance,
                parsed.sensor_data.packet_num, parsed.rssi, parsed.snr);
                cx.shared.last_packet.lock(| last_pkt |
                { * last_pkt = Some(parsed); });
                cx.shared.packets_received.lock(| count | { * count += 1; });
                cx.shared.lora_uart.lock(| uart |
                { send_ack(uart, parsed.sensor_data.packet_num, true); }); let
                timestamp = cx.shared.uptime_ms.lock(| t | * t); let total =
                cx.shared.packets_received.lock(| c | * c); let errors =
                cx.shared.crc_errors.lock(| e | * e); let gw_temp =
                cx.shared.gateway_temp.lock(| t | * t); let gw_humidity =
                cx.shared.gateway_humidity.lock(| h | * h); let json =
                format_json_telemetry(& parsed, timestamp, total, errors,
                gw_temp, gw_humidity);
                cx.shared.vcp_uart.lock(| uart |
                {
                    for byte in json.as_bytes()
                    { let _ = nb :: block! (uart.write(*byte)); }
                }); defmt :: info! ("JSON sent via VCP: {}", json.as_str());
            } else
            {
                defmt :: warn! ("Failed to parse binary message");
                cx.shared.crc_errors.lock(| errors | * errors += 1);
            } cx.local.rx_buffer.clear();
        }
    } #[doc = " RTIC shared resource struct"] struct Shared
    {
        lora_uart : Serial < pac :: UART4 > , vcp_uart : Serial < pac ::
        USART2 > , display : LoraDisplay, last_packet : Option < ParsedMessage
        > , packets_received : u32, crc_errors : u32, sht3x : Option < SHT3x <
        I2cProxy, ShtDelay > > , sht3x_skip_reads : u8, gateway_temp : Option
        < f32 > , gateway_humidity : Option < f32 > , uptime_ms : u32,
    } #[doc = " RTIC local resource struct"] struct Local
    {
        led : Pin < 'A', 5, Output > , timer : CounterHz < pac :: TIM2 > ,
        rx_buffer : Vec < u8, RX_BUFFER_SIZE > ,
    } #[doc = r" Monotonics used by the system"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_Monotonics();
    #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct __rtic_internal_init_Context <
    'a >
    {
        #[doc = r" Core (Cortex-M) peripherals"] pub core : rtic :: export ::
        Peripherals, #[doc = r" Device peripherals"] pub device :
        stm32f4xx_hal :: pac :: Peripherals,
        #[doc = r" Critical section token for init"] pub cs : rtic :: export
        :: CriticalSection < 'a > ,
    } impl < 'a > __rtic_internal_init_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(core : rtic :: export :: Peripherals,) -> Self
        {
            __rtic_internal_init_Context
            {
                device : stm32f4xx_hal :: pac :: Peripherals :: steal(), cs :
                rtic :: export :: CriticalSection :: new(), core,
            }
        }
    } #[allow(non_snake_case)] #[doc = " Initialization function"] pub mod
    init
    {
        #[doc(inline)] pub use super :: __rtic_internal_Monotonics as
        Monotonics; #[doc(inline)] pub use super ::
        __rtic_internal_init_Context as Context;
    } mod shared_resources
    {
        use rtic :: export :: Priority; #[doc(hidden)]
        #[allow(non_camel_case_types)] pub struct
        lora_uart_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > lora_uart_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { lora_uart_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        vcp_uart_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > vcp_uart_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { vcp_uart_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        display_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > display_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { display_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        last_packet_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        last_packet_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { last_packet_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        packets_received_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        packets_received_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { packets_received_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        crc_errors_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        crc_errors_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { crc_errors_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        sht3x_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > sht3x_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { sht3x_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        sht3x_skip_reads_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        sht3x_skip_reads_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { sht3x_skip_reads_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        gateway_temp_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        gateway_temp_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { gateway_temp_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        gateway_humidity_that_needs_to_be_locked < 'a >
        { priority : & 'a Priority, } impl < 'a >
        gateway_humidity_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { gateway_humidity_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        } #[doc(hidden)] #[allow(non_camel_case_types)] pub struct
        uptime_ms_that_needs_to_be_locked < 'a > { priority : & 'a Priority, }
        impl < 'a > uptime_ms_that_needs_to_be_locked < 'a >
        {
            #[inline(always)] pub unsafe fn new(priority : & 'a Priority) ->
            Self { uptime_ms_that_needs_to_be_locked { priority } }
            #[inline(always)] pub unsafe fn priority(& self) -> & Priority
            { self.priority }
        }
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Local resources `tim2_handler` has access to"] pub struct
    __rtic_internal_tim2_handlerLocalResources < 'a >
    {
        #[doc = " Local resource `led`"] pub led : & 'a mut Pin < 'A', 5,
        Output > , #[doc = " Local resource `timer`"] pub timer : & 'a mut
        CounterHz < pac :: TIM2 > ,
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Shared resources `tim2_handler` has access to"] pub struct
    __rtic_internal_tim2_handlerSharedResources < 'a >
    {
        #[doc =
        " Resource proxy resource `display`. Use method `.lock()` to gain access"]
        pub display : shared_resources :: display_that_needs_to_be_locked < 'a
        > ,
        #[doc =
        " Resource proxy resource `last_packet`. Use method `.lock()` to gain access"]
        pub last_packet : shared_resources ::
        last_packet_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `packets_received`. Use method `.lock()` to gain access"]
        pub packets_received : shared_resources ::
        packets_received_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `sht3x`. Use method `.lock()` to gain access"]
        pub sht3x : shared_resources :: sht3x_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `sht3x_skip_reads`. Use method `.lock()` to gain access"]
        pub sht3x_skip_reads : shared_resources ::
        sht3x_skip_reads_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `gateway_temp`. Use method `.lock()` to gain access"]
        pub gateway_temp : shared_resources ::
        gateway_temp_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `gateway_humidity`. Use method `.lock()` to gain access"]
        pub gateway_humidity : shared_resources ::
        gateway_humidity_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `uptime_ms`. Use method `.lock()` to gain access"]
        pub uptime_ms : shared_resources :: uptime_ms_that_needs_to_be_locked
        < 'a > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct
    __rtic_internal_tim2_handler_Context < 'a >
    {
        #[doc = r" Local Resources this task has access to"] pub local :
        tim2_handler :: LocalResources < 'a > ,
        #[doc = r" Shared Resources this task has access to"] pub shared :
        tim2_handler :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_tim2_handler_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_tim2_handler_Context
            {
                local : tim2_handler :: LocalResources :: new(), shared :
                tim2_handler :: SharedResources :: new(priority),
            }
        }
    } #[allow(non_snake_case)] #[doc = " Hardware task"] pub mod tim2_handler
    {
        #[doc(inline)] pub use super ::
        __rtic_internal_tim2_handlerLocalResources as LocalResources;
        #[doc(inline)] pub use super ::
        __rtic_internal_tim2_handlerSharedResources as SharedResources;
        #[doc(inline)] pub use super :: __rtic_internal_tim2_handler_Context
        as Context;
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Local resources `uart4_handler` has access to"] pub struct
    __rtic_internal_uart4_handlerLocalResources < 'a >
    {
        #[doc = " Local resource `rx_buffer`"] pub rx_buffer : & 'a mut Vec <
        u8, RX_BUFFER_SIZE > ,
    } #[allow(non_snake_case)] #[allow(non_camel_case_types)]
    #[doc = " Shared resources `uart4_handler` has access to"] pub struct
    __rtic_internal_uart4_handlerSharedResources < 'a >
    {
        #[doc =
        " Resource proxy resource `lora_uart`. Use method `.lock()` to gain access"]
        pub lora_uart : shared_resources :: lora_uart_that_needs_to_be_locked
        < 'a > ,
        #[doc =
        " Resource proxy resource `vcp_uart`. Use method `.lock()` to gain access"]
        pub vcp_uart : shared_resources :: vcp_uart_that_needs_to_be_locked <
        'a > ,
        #[doc =
        " Resource proxy resource `last_packet`. Use method `.lock()` to gain access"]
        pub last_packet : shared_resources ::
        last_packet_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `packets_received`. Use method `.lock()` to gain access"]
        pub packets_received : shared_resources ::
        packets_received_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `crc_errors`. Use method `.lock()` to gain access"]
        pub crc_errors : shared_resources ::
        crc_errors_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `gateway_temp`. Use method `.lock()` to gain access"]
        pub gateway_temp : shared_resources ::
        gateway_temp_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `gateway_humidity`. Use method `.lock()` to gain access"]
        pub gateway_humidity : shared_resources ::
        gateway_humidity_that_needs_to_be_locked < 'a > ,
        #[doc =
        " Resource proxy resource `uptime_ms`. Use method `.lock()` to gain access"]
        pub uptime_ms : shared_resources :: uptime_ms_that_needs_to_be_locked
        < 'a > ,
    } #[doc = r" Execution context"] #[allow(non_snake_case)]
    #[allow(non_camel_case_types)] pub struct
    __rtic_internal_uart4_handler_Context < 'a >
    {
        #[doc = r" Local Resources this task has access to"] pub local :
        uart4_handler :: LocalResources < 'a > ,
        #[doc = r" Shared Resources this task has access to"] pub shared :
        uart4_handler :: SharedResources < 'a > ,
    } impl < 'a > __rtic_internal_uart4_handler_Context < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_uart4_handler_Context
            {
                local : uart4_handler :: LocalResources :: new(), shared :
                uart4_handler :: SharedResources :: new(priority),
            }
        }
    } #[allow(non_snake_case)] #[doc = " Hardware task"] pub mod uart4_handler
    {
        #[doc(inline)] pub use super ::
        __rtic_internal_uart4_handlerLocalResources as LocalResources;
        #[doc(inline)] pub use super ::
        __rtic_internal_uart4_handlerSharedResources as SharedResources;
        #[doc(inline)] pub use super :: __rtic_internal_uart4_handler_Context
        as Context;
    } #[doc = r" App module"] #[allow(non_camel_case_types)]
    #[allow(non_upper_case_globals)] #[doc(hidden)]
    #[link_section = ".uninit.rtic0"] static
    __rtic_internal_shared_resource_lora_uart : rtic :: RacyCell < core :: mem
    :: MaybeUninit < Serial < pac :: UART4 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: lora_uart_that_needs_to_be_locked < 'a >
    {
        type T = Serial < pac :: UART4 > ; #[inline(always)] fn lock <
        RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut Serial < pac :: UART4 >) ->
        RTIC_INTERNAL_R) -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_lora_uart.get_mut() as *
                mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic1"] static
    __rtic_internal_shared_resource_vcp_uart : rtic :: RacyCell < core :: mem
    :: MaybeUninit < Serial < pac :: USART2 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: vcp_uart_that_needs_to_be_locked < 'a >
    {
        type T = Serial < pac :: USART2 > ; #[inline(always)] fn lock <
        RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut Serial < pac :: USART2 >) ->
        RTIC_INTERNAL_R) -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_vcp_uart.get_mut() as *
                mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic2"] static
    __rtic_internal_shared_resource_display : rtic :: RacyCell < core :: mem
    :: MaybeUninit < LoraDisplay >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: display_that_needs_to_be_locked < 'a >
    {
        type T = LoraDisplay; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut LoraDisplay) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_display.get_mut() as *
                mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic3"] static
    __rtic_internal_shared_resource_last_packet : rtic :: RacyCell < core ::
    mem :: MaybeUninit < Option < ParsedMessage > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: last_packet_that_needs_to_be_locked < 'a >
    {
        type T = Option < ParsedMessage > ; #[inline(always)] fn lock <
        RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut Option < ParsedMessage >) ->
        RTIC_INTERNAL_R) -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_last_packet.get_mut() as
                * mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic4"] static
    __rtic_internal_shared_resource_packets_received : rtic :: RacyCell < core
    :: mem :: MaybeUninit < u32 >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: packets_received_that_needs_to_be_locked < 'a >
    {
        type T = u32; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut u32) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_packets_received.get_mut()
                as * mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic5"] static
    __rtic_internal_shared_resource_crc_errors : rtic :: RacyCell < core ::
    mem :: MaybeUninit < u32 >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: crc_errors_that_needs_to_be_locked < 'a >
    {
        type T = u32; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut u32) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_crc_errors.get_mut() as *
                mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic6"] static
    __rtic_internal_shared_resource_sht3x : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Option < SHT3x < I2cProxy, ShtDelay > > >> = rtic ::
    RacyCell :: new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic
    :: Mutex for shared_resources :: sht3x_that_needs_to_be_locked < 'a >
    {
        type T = Option < SHT3x < I2cProxy, ShtDelay > > ; #[inline(always)]
        fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl
        FnOnce(& mut Option < SHT3x < I2cProxy, ShtDelay > >) ->
        RTIC_INTERNAL_R) -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_sht3x.get_mut() as * mut
                _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic7"] static
    __rtic_internal_shared_resource_sht3x_skip_reads : rtic :: RacyCell < core
    :: mem :: MaybeUninit < u8 >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: sht3x_skip_reads_that_needs_to_be_locked < 'a >
    {
        type T = u8; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut u8) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_sht3x_skip_reads.get_mut()
                as * mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic8"] static
    __rtic_internal_shared_resource_gateway_temp : rtic :: RacyCell < core ::
    mem :: MaybeUninit < Option < f32 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: gateway_temp_that_needs_to_be_locked < 'a >
    {
        type T = Option < f32 > ; #[inline(always)] fn lock < RTIC_INTERNAL_R
        >
        (& mut self, f : impl FnOnce(& mut Option < f32 >) -> RTIC_INTERNAL_R)
        -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_gateway_temp.get_mut() as
                * mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic9"] static
    __rtic_internal_shared_resource_gateway_humidity : rtic :: RacyCell < core
    :: mem :: MaybeUninit < Option < f32 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: gateway_humidity_that_needs_to_be_locked < 'a >
    {
        type T = Option < f32 > ; #[inline(always)] fn lock < RTIC_INTERNAL_R
        >
        (& mut self, f : impl FnOnce(& mut Option < f32 >) -> RTIC_INTERNAL_R)
        -> RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_gateway_humidity.get_mut()
                as * mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic10"] static
    __rtic_internal_shared_resource_uptime_ms : rtic :: RacyCell < core :: mem
    :: MaybeUninit < u32 >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); impl < 'a > rtic :: Mutex for
    shared_resources :: uptime_ms_that_needs_to_be_locked < 'a >
    {
        type T = u32; #[inline(always)] fn lock < RTIC_INTERNAL_R >
        (& mut self, f : impl FnOnce(& mut u32) -> RTIC_INTERNAL_R) ->
        RTIC_INTERNAL_R
        {
            #[doc = r" Priority ceiling"] const CEILING : u8 = 1u8; unsafe
            {
                rtic :: export ::
                lock(__rtic_internal_shared_resource_uptime_ms.get_mut() as *
                mut _, self.priority(), CEILING, stm32f4xx_hal :: pac ::
                NVIC_PRIO_BITS, & __rtic_internal_MASKS, f,)
            }
        }
    } #[doc(hidden)] #[allow(non_upper_case_globals)] const
    __rtic_internal_MASK_CHUNKS : usize = rtic :: export ::
    compute_mask_chunks([stm32f4xx_hal :: pac :: Interrupt :: TIM2 as u32,
    stm32f4xx_hal :: pac :: Interrupt :: UART4 as u32]); #[doc(hidden)]
    #[allow(non_upper_case_globals)] const __rtic_internal_MASKS :
    [rtic :: export :: Mask < __rtic_internal_MASK_CHUNKS > ; 3] =
    [rtic :: export ::
    create_mask([stm32f4xx_hal :: pac :: Interrupt :: TIM2 as u32,
    stm32f4xx_hal :: pac :: Interrupt :: UART4 as u32]), rtic :: export ::
    create_mask([]), rtic :: export :: create_mask([])];
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic11"] static
    __rtic_internal_local_resource_led : rtic :: RacyCell < core :: mem ::
    MaybeUninit < Pin < 'A', 5, Output > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic12"] static
    __rtic_internal_local_resource_timer : rtic :: RacyCell < core :: mem ::
    MaybeUninit < CounterHz < pac :: TIM2 > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit());
    #[allow(non_camel_case_types)] #[allow(non_upper_case_globals)]
    #[doc(hidden)] #[link_section = ".uninit.rtic13"] static
    __rtic_internal_local_resource_rx_buffer : rtic :: RacyCell < core :: mem
    :: MaybeUninit < Vec < u8, RX_BUFFER_SIZE > >> = rtic :: RacyCell ::
    new(core :: mem :: MaybeUninit :: uninit()); #[allow(non_snake_case)]
    #[no_mangle] #[doc = " User HW task ISR trampoline for tim2_handler"]
    unsafe fn TIM2()
    {
        const PRIORITY : u8 = 1u8; rtic :: export ::
        run(PRIORITY, ||
        {
            tim2_handler(tim2_handler :: Context ::
            new(& rtic :: export :: Priority :: new(PRIORITY)))
        });
    } impl < 'a > __rtic_internal_tim2_handlerLocalResources < 'a >
    {
        #[inline(always)] #[doc(hidden)] pub unsafe fn new() -> Self
        {
            __rtic_internal_tim2_handlerLocalResources
            {
                led : & mut *
                (& mut *
                __rtic_internal_local_resource_led.get_mut()).as_mut_ptr(),
                timer : & mut *
                (& mut *
                __rtic_internal_local_resource_timer.get_mut()).as_mut_ptr(),
            }
        }
    } impl < 'a > __rtic_internal_tim2_handlerSharedResources < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_tim2_handlerSharedResources
            {
                #[doc(hidden)] display : shared_resources ::
                display_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] last_packet : shared_resources ::
                last_packet_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] packets_received : shared_resources ::
                packets_received_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] sht3x : shared_resources ::
                sht3x_that_needs_to_be_locked :: new(priority), #[doc(hidden)]
                sht3x_skip_reads : shared_resources ::
                sht3x_skip_reads_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] gateway_temp : shared_resources ::
                gateway_temp_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] gateway_humidity : shared_resources ::
                gateway_humidity_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] uptime_ms : shared_resources ::
                uptime_ms_that_needs_to_be_locked :: new(priority),
            }
        }
    } #[allow(non_snake_case)] #[no_mangle]
    #[doc = " User HW task ISR trampoline for uart4_handler"] unsafe fn
    UART4()
    {
        const PRIORITY : u8 = 1u8; rtic :: export ::
        run(PRIORITY, ||
        {
            uart4_handler(uart4_handler :: Context ::
            new(& rtic :: export :: Priority :: new(PRIORITY)))
        });
    } impl < 'a > __rtic_internal_uart4_handlerLocalResources < 'a >
    {
        #[inline(always)] #[doc(hidden)] pub unsafe fn new() -> Self
        {
            __rtic_internal_uart4_handlerLocalResources
            {
                rx_buffer : & mut *
                (& mut *
                __rtic_internal_local_resource_rx_buffer.get_mut()).as_mut_ptr(),
            }
        }
    } impl < 'a > __rtic_internal_uart4_handlerSharedResources < 'a >
    {
        #[doc(hidden)] #[inline(always)] pub unsafe fn
        new(priority : & 'a rtic :: export :: Priority) -> Self
        {
            __rtic_internal_uart4_handlerSharedResources
            {
                #[doc(hidden)] lora_uart : shared_resources ::
                lora_uart_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] vcp_uart : shared_resources ::
                vcp_uart_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] last_packet : shared_resources ::
                last_packet_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] packets_received : shared_resources ::
                packets_received_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] crc_errors : shared_resources ::
                crc_errors_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] gateway_temp : shared_resources ::
                gateway_temp_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] gateway_humidity : shared_resources ::
                gateway_humidity_that_needs_to_be_locked :: new(priority),
                #[doc(hidden)] uptime_ms : shared_resources ::
                uptime_ms_that_needs_to_be_locked :: new(priority),
            }
        }
    } #[doc(hidden)] mod rtic_ext
    {
        use super :: * ; #[no_mangle] unsafe extern "C" fn main() -> !
        {
            rtic :: export :: assert_send :: < Serial < pac :: UART4 > > ();
            rtic :: export :: assert_send :: < Serial < pac :: USART2 > > ();
            rtic :: export :: assert_send :: < LoraDisplay > (); rtic ::
            export :: assert_send :: < Option < ParsedMessage > > (); rtic ::
            export :: assert_send :: < u32 > (); rtic :: export :: assert_send
            :: < Option < SHT3x < I2cProxy, ShtDelay > > > (); rtic :: export
            :: assert_send :: < u8 > (); rtic :: export :: assert_send :: <
            Option < f32 > > (); rtic :: export :: assert_send :: < Pin < 'A',
            5, Output > > (); rtic :: export :: assert_send :: < CounterHz <
            pac :: TIM2 > > (); rtic :: export :: assert_send :: < Vec < u8,
            RX_BUFFER_SIZE > > (); const _CONST_CHECK : () =
            {
                if ! rtic :: export :: have_basepri()
                {
                    if (stm32f4xx_hal :: pac :: Interrupt :: TIM2 as usize) >=
                    (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        :: core :: panic!
                        ("An interrupt out of range is used while in armv6 or armv8m.base");
                    } if (stm32f4xx_hal :: pac :: Interrupt :: UART4 as usize)
                    >= (__rtic_internal_MASK_CHUNKS * 32)
                    {
                        :: core :: panic!
                        ("An interrupt out of range is used while in armv6 or armv8m.base");
                    }
                } else {}
            }; let _ = _CONST_CHECK; rtic :: export :: interrupt :: disable();
            let mut core : rtic :: export :: Peripherals = rtic :: export ::
            Peripherals :: steal().into(); const _ : () = if
            (1 << stm32f4xx_hal :: pac :: NVIC_PRIO_BITS) < 1u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'TIM2' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: TIM2, rtic :: export ::
            logical2hw(1u8, stm32f4xx_hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: TIM2); const _ : () = if
            (1 << stm32f4xx_hal :: pac :: NVIC_PRIO_BITS) < 1u8 as usize
            {
                :: core :: panic!
                ("Maximum priority used by interrupt vector 'UART4' is more than supported by hardware");
            };
            core.NVIC.set_priority(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: UART4, rtic :: export ::
            logical2hw(1u8, stm32f4xx_hal :: pac :: NVIC_PRIO_BITS),); rtic ::
            export :: NVIC ::
            unmask(you_must_enable_the_rt_feature_for_the_pac_in_your_cargo_toml
            :: interrupt :: UART4); #[inline(never)] fn __rtic_init_resources
            < F > (f : F) where F : FnOnce() { f(); }
            __rtic_init_resources(||
            {
                let (shared_resources, local_resources, mut monotonics) =
                init(init :: Context :: new(core.into()));
                __rtic_internal_shared_resource_lora_uart.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.lora_uart));
                __rtic_internal_shared_resource_vcp_uart.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.vcp_uart));
                __rtic_internal_shared_resource_display.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.display));
                __rtic_internal_shared_resource_last_packet.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.last_packet));
                __rtic_internal_shared_resource_packets_received.get_mut().write(core
                :: mem :: MaybeUninit ::
                new(shared_resources.packets_received));
                __rtic_internal_shared_resource_crc_errors.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.crc_errors));
                __rtic_internal_shared_resource_sht3x.get_mut().write(core ::
                mem :: MaybeUninit :: new(shared_resources.sht3x));
                __rtic_internal_shared_resource_sht3x_skip_reads.get_mut().write(core
                :: mem :: MaybeUninit ::
                new(shared_resources.sht3x_skip_reads));
                __rtic_internal_shared_resource_gateway_temp.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.gateway_temp));
                __rtic_internal_shared_resource_gateway_humidity.get_mut().write(core
                :: mem :: MaybeUninit ::
                new(shared_resources.gateway_humidity));
                __rtic_internal_shared_resource_uptime_ms.get_mut().write(core
                :: mem :: MaybeUninit :: new(shared_resources.uptime_ms));
                __rtic_internal_local_resource_led.get_mut().write(core :: mem
                :: MaybeUninit :: new(local_resources.led));
                __rtic_internal_local_resource_timer.get_mut().write(core ::
                mem :: MaybeUninit :: new(local_resources.timer));
                __rtic_internal_local_resource_rx_buffer.get_mut().write(core
                :: mem :: MaybeUninit :: new(local_resources.rx_buffer)); rtic
                :: export :: interrupt :: enable();
            }); loop { rtic :: export :: nop() }
        }
    }
}