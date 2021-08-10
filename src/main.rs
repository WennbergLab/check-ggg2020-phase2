use std::collections::HashMap;
use std::env;
use float_cmp::{ApproxEq,F32Margin};
use lazy_static::lazy_static;
use regex::Regex;

const ADCF_TABLE: &'static str = " Gas         ADCF      ADCF_Err  g    p
\"xco2_6220\"  -0.00903  0.00025   15   4
\"xco2_6339\"  -0.00512  0.00025   45   5
\"xlco2_4852\"  0.00008  0.00018  -45   1
\"xwco2_6073\" -0.00235  0.00016  -45   1
\"xwco2_6500\" -0.00970  0.00026   45   5
\"xch4_5938\"  -0.00971  0.00046   25   4
\"xch4_6002\"  -0.00602  0.00053  -5    2
\"xch4_6076\"  -0.00594  0.00044   15   3
\"xn2o_4395\"   0.00523  0.00054  -5    2
\"xn2o_4430\"   0.00426  0.00042   13   3
\"xn2o_4719\"  -0.00267  0.00056  -15   2
\"xco_4233\"    0.00000  0.00000   13   3
\"xco_4290\"    0.00000  0.00000   13   3
\"xluft_6146\"  0.00053  0.00017  -45   1";

#[derive(Debug)]
struct Adcf {
    window: &'static str,
    adcf: f32,
    err: f32,
    g: i32,
    p: i32
}


fn read_adcf_table() -> HashMap<&'static str, Adcf> {
    let mut adcfs = HashMap::new();
    let mut first_line = true;
    for line in ADCF_TABLE.split("\n") {
        if first_line {
            first_line = false;
            continue;
        }

        let parts: Vec<&'static str> = line.split_whitespace().into_iter().collect();
        let window = parts[0].strip_prefix('"').unwrap().strip_suffix('"').unwrap();
        let s = Adcf{
            window: window, 
            adcf: parts[1].parse::<f32>().unwrap(), 
            err: parts[2].parse::<f32>().unwrap(), 
            g: parts[3].parse::<i32>().unwrap(), 
            p: parts[4].parse::<i32>().unwrap()
        };
        adcfs.insert(window, s);
    }

    return adcfs;
}


const AICF_TABLE: &'static str = " Gas     AICF  AICF_Err  WMO_Scale
\"xco2\"   1.0101  0.0005  \"WMO CO2 X2007\"
\"xwco2\"  1.0008  0.0005  \"WMO CO2 X2007\"
\"xlco2\"  1.0014  0.0007  \"WMO CO2 X2007\"
\"xch4\"   1.0031  0.0014  \"WMO CH4 X2004\"
\"xn2o\"   0.9822  0.0105  \"NOAA 2006A\"
\"xco\"    1.0000  0.0526  \"N/A\"
\"xh2o\"   0.9882  0.0157  \"N/A\"
\"xluft\"  1.0000  0.0000  \"N/A\"";


#[derive(Debug)]
struct Aicf {
    gas: &'static str,
    aicf: f32,
    err: f32
}


fn read_aicf_table() -> HashMap<&'static str, Aicf> {
    let mut aicfs = HashMap::new();
    let mut first_line = true;
    for line in AICF_TABLE.split("\n") {
        if first_line {
            first_line = false;
            continue;
        }

        let parts: Vec<&'static str> = line.split_whitespace().into_iter().collect();
        let gas = parts[0].strip_prefix('"').unwrap().strip_suffix('"').unwrap();
        let s = Aicf{
            gas: gas, 
            aicf: parts[1].parse::<f32>().unwrap(), 
            err: parts[2].parse::<f32>().unwrap(), 
        };
        aicfs.insert(gas, s);
    }

    return aicfs;
}


const WINDOWS_TABLE: &'static str = " Center   Width MIT A I F  Parameters_to_ fit  Bias      Gases_to_fit
6146.90   1.60   0 1 1 0                     sf=1.000 : luft
4038.95   0.32  15 1 1 0  ncbf=2  fs  so     sf=1.000 : hf  h2o
4565.20   2.50  15 1 1 0  ncbf=2  fs  sg     sf=1.006 : h2o  co2 ch4
4570.35   3.10  15 1 1 0  ncbf=2  fs  sg     sf=0.994 : h2o  co2  ch4
4571.75   2.50  15 1 1 0  ncbf=2  fs  so     sf=0.996 : h2o  co2 ch4
4576.85   1.90  15 1 1 0  ncbf=2  fs  so     sf=1.009 : h2o  ch4
4598.69  10.78  20 1 1 0  ncbf=2  fs  sg     sf=1.003 : h2o  ch4  co2  n2o
4611.05   2.20  15 1 1 0  ncbf=2  fs  so     sf=0.993 : h2o  ch4  co2  n2o
4622.00   2.30  15 1 1 0  ncbf=2  fs  so     sf=1.001 : h2o  co2  n2o
4631.55   1.40  20 1 1 0  ncbf=2  fs  so     sf=0.990 : h2o
4699.55   4.00  15 1 1 0  ncbf=2  fs  so     sf=1.001 : h2o  co2  n2o
4734.60   7.30  20 1 1 0  ncbf=2  fs  sg     sf=1.000 : h2o  co2  n2o
4761.15  10.70  20 1 1 0  ncbf=2  fs  so     sf=1.000 : h2o  co2
6076.90   3.85  15 1 1 0  ncbf=2  fs  sg     sf=1.018 : h2o  ch4 hdo co2
6099.35   0.95  15 1 1 0  ncbf=2  fs  so     sf=1.001 : h2o  co2
6125.85   1.45  15 1 1 0  ncbf=2  fs  sg     sf=1.007 : h2o  hdo co2 ch4
:6177.30   0.83  15 1 1 0  ncbf=2  fs  so     sf=1.000 : h2o  hdo co2 ch4
6177.51   1.26  15 1 1 0  ncbf=2  fs  sg     sf=1.005 : h2o  hdo co2 ch4
:6219.00   7.00  15 1 1 0  ncbf=2  fs  so     sf=1.000 : h2o  co2 ch4
:6244.40   7.20  15 1 1 0  ncbf=2  fs  so     sf=1.000 : h2o  co2 hdo
6255.95   3.60  15 1 1 0  ncbf=2  fs  sg  nv sf=0.994 : h2o  co2 hdo
6301.35   7.90  15 1 1 0  ncbf=2  fs  sg     sf=0.999 : h2o  co2 hdo
6392.45   3.10  15 1 1 0  ncbf=2  fs  sg     sf=1.016 : h2o  hdo
6401.15   1.15  15 1 1 0  ncbf=2  fs  sg     sf=1.014 : h2o  hdo co2
6469.60   3.50  15 1 1 0  ncbf=2  fs  sg     sf=0.989 : h2o  co2 hdo
4054.90   3.00  15 1 1 0  ncbf=2  fs  sg     sf=1.020 : th2o  ch4  n2o  hdo  
4255.74   2.82  15 1 1 0  ncbf=2  fs  so     sf=1.005 : th2o  ch4  co  hdo
4325.50   3.02  15 1 1 0  ncbf=2  fs  sg     sf=1.012 : th2o  ch4  co  hdo
4493.90   1.80  15 1 1 0  ncbf=2  fs  so     sf=1.000 : th2o  ch4 
4516.71   2.42  15 1 1 0  ncbf=2  fs  so     sf=1.002 : th2o  ch4
4524.10   2.00  15 1 1 0  ncbf=2  fs  so     sf=0.999 : th2o  ch4  co2 
:4596.65   1.40  15 1 1 0  ncbf=2  fs  so     sf=1.000 : th2o  ch4  co2  n2o   
4633.64   1.82  15 1 1 0  ncbf=2  fs  sg     sf=0.987 : th2o  co2  n2o
4054.60   3.30  15 1 1 0  ncbf=2  fs  sg     sf=0.995 : hdo  h2o  ch4
4067.60   8.80  15 1 1 0  ncbf=2  fs  sg     sf=0.992 : hdo  h2o  ch4
4116.10   8.00  15 1 1 0  ncbf=2  fs  sg     sf=0.992 : hdo  h2o  ch4
4212.45   1.90  15 1 1 0  ncbf=2  fs  so     sf=1.002 : hdo  h2o  ch4
4232.50  11.00  15 1 1 0  ncbf=2  fs  sg     sf=0.996 : hdo  h2o  ch4  co
:4261.70   9.10  15 1 1 0  ncbf=2  fs  sg     sf=1.000 : hdo  h2o  ch4  co
6330.05  45.50  15 1 1 0  ncbf=4  fs  sg     sf=0.990 : hdo  h2o  co2
6377.40  50.20  15 1 1 0  ncbf=4  fs  sg  nv sf=1.009 : hdo  h2o  co2
6458.10  41.40  15 1 1 0  ncbf=4  fs  sg     sf=1.014 : hdo  h2o  co2 
:4233.10  48.40  15 1 1 0  ncbf=3  fs  sg     sf=1.000 : co  ch4 h2o hdo
4290.50  56.60  15 1 1 0  ncbf=4  fs  sg     sf=1.000 : co  ch4 h2o hdo
4395.20  43.40  15 1 1 0  ncbf=4  fs  sg     sf=0.993 : n2o ch4 h2o hdo
4430.10  23.10  15 1 1 0  ncbf=2  fs  sg     sf=0.995 : n2o ch4 h2o hdo co2
4719.50  73.10  15 1 1 0  ncbf=3  fs  sg     sf=1.008 : n2o ch4 h2o co2
5938.00 116.00  15 1 1 0  ncbf=4  fs  sg  nv sf=1.005 : ch4 co2 h2o n2o
6002.00  11.10  15 1 1 0  ncbf=2  fs  sg  nv sf=1.000 : ch4 co2 h2o hdo
6076.00 138.00  15 1 1 0  ncbf=5  fs  sg  nv sf=0.995 : ch4 co2 h2o hdo
:6002.50 268.20  15 1 1 0  ncbf=6  fs  sg  nv sf=1.000 : 2ch4 ch4 co2 h2o hdo
4852.87  86.26  15 1 1 0  ncbf=3  fs  sg  nv sf=1.000 : lco2 2co2 3co2 4co2 h2o hdo 
4852.20  87.60  15 1 1 0  ncbf=3  fs  sg  nv          : zco2 h2o hdo
4852.20  87.60  15 1 1 0  ncbf=3  fs  sg  nv  zo      : zco2 h2o hdo
:2644.35 100.10  15 1 1 0  ncbf=4  fs  sg  cf          : fco2  h2o  hdo  ch4
6154.70  75.50  15 1 1 0  ncbf=4  fs  sg  cf          : fco2 h2o hdo ch4
:12881.20  31.60  15 1 1 0  ncbf=3  fs  sg  cf          : fco2 h2o o2
6073.50  63.40  15 1 1 0  ncbf=2  fs  sg  nv sf=1.000 : wco2 h2o ch4
:6500.40  58.00  15 1 1 0  ncbf=2  fs  sg  nv sf=1.000 : wco2 h2o hdo 
6220.00  80.00  15 1 1 0  ncbf=3  fs  sg  nv sf=1.001 : co2 h2o hdo ch4
6339.50  85.00  15 1 1 0  ncbf=3  fs  sg  nv sf=0.999 : co2 h2o hdo
7885.00 240.00  15 1 1 0  ncbf=5  fs  sg  nv sf=1.000 : o2 0o2 h2o hf co2 hdo
:13082.50 225.00 15 1 1 0  ncbf=2  fs  sg     sf=1.000 : ao2
:14465.00 234.00 15 1 1 0  ncbf=2  fs  sg     sf=1.000 : bo2 h2o    
:5577.30   0.40  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
:5597.80   0.40  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
5625.02   0.29  15 0 1 0  ncbf=2  fs  so     sf=1.002 : hcl h2o ch4
:5642.90   1.50  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
:5683.57   0.36  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o
5687.65   1.10  15 0 1 0  ncbf=2  fs  sg     sf=1.001 : hcl h2o ch4
5702.00   0.70  15 0 1 0  ncbf=2  fs  sg     sf=0.989 : hcl h2o ch4
:5706.20   0.50  15 0 1 0  ncbf=2  fs  sg     sf=1.000 : hcl h2o ch4
:5719.12   2.26  15 0 1 0  ncbf=2  fs  sg     sf=1.000 : hcl h2o ch4
5735.05   0.52  15 0 1 0  ncbf=2  fs  sg     sf=0.998 : hcl h2o ch4
5739.25   1.50  15 0 1 0  ncbf=2  fs  sg     sf=1.003 : hcl h2o ch4
:5743.00 125.02  15 0 1 0  ncbf=6  fs  sg  zo sf=1.000 : hcl h2o ch4
:5749.80   0.60  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
:5754.00   0.80  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
:5763.20   0.68  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4
:5767.35   1.70  15 0 1 0  ncbf=2  fs  sg     sf=1.000 : hcl h2o ch4
:5779.50   1.00  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4 
:5790.45   0.90  15 0 1 0  ncbf=2  fs  so     sf=1.000 : hcl h2o ch4";

#[derive(Debug)]
struct Window {
    center: i32,
    gas: &'static str,
    sf: f32
}


fn read_windows_table() -> (HashMap<String, Window>, Vec<String>) {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"sf=(\d\.\d+)").unwrap();
    }
    let mut windows = HashMap::new();
    let mut skipped_windows = Vec::new();
    let mut first_line = true;
    for line in WINDOWS_TABLE.split("\n") {
        if first_line {
            first_line = false;
            
        }else if line.starts_with(':') {
            let (win_name, _, _) = get_window_name(&line[1..]);
            skipped_windows.push(win_name);
        }else{

            let (win_name, main_gas, center_str) = get_window_name(line);
            let sf = if let Some(caps) = RE.captures(line) {
                let v = caps.get(1).unwrap().as_str();
                v.parse::<f32>().unwrap()
            }else{
                1.0
            };
            
            let s = Window{
                center: center_str.parse::<i32>().unwrap(),
                gas: main_gas,
                sf: sf
            };

            println!("{} = {:?}", win_name, s);
            windows.insert(win_name, s);
        }
    }

    println!("skipped windows = {:?}", skipped_windows);
    return (windows, skipped_windows);
}

fn get_window_name(table_line: &'static str) -> (String, &'static str, &'static str) {
    let tmp: Vec<&'static str> = table_line.split(':').into_iter().collect();
    let cmd: Vec<&'static str> = tmp[0].split_whitespace().into_iter().collect();
    let gases: Vec<&'static str> = tmp[1].split_whitespace().into_iter().collect();

    let center_str = cmd[0].split('.').next().unwrap();
    let main_gas = gases[0];
    let window_name = format!("{}_{}", main_gas, center_str);

    return (window_name, main_gas, center_str)
}


// ************* //
// GENERAL UTILS //
// ************* //

/* Verbosity levels:

   -1 = no messages, just indicate by exit code
    0 = just print summary message at the end
    1 = print for each category
    2 = print for each gas/window
    3 = print for each variable
 */

fn _get_var<'a>(nch: &'a netcdf::File, varname: &str) -> Result<netcdf::Variable<'a>, String> {
    match nch.variable(varname) {
        Some(v) => return Ok(v),
        None => return Err(format!("Could not read variable '{}'", varname))
    }
}

fn _print_variable_results(varname: &str, n_total: usize, n_wrong: usize, verbosity: i8) -> bool {
    let is_ok = n_wrong == 0;
    if is_ok {
        if verbosity >= 3 {
            println!("    - PASS: {}", varname);
        }
    } else {
        if verbosity >= 3 {
            let percent = n_wrong as f32 / n_total as f32 * 100.0;
            println!("    - FAIL: {}/{} ({:.2}%) of {} have incorrect values", n_wrong, n_total, percent, varname);
        }
    }

    return is_ok;
}

fn _all_equal_float(var: &netcdf::Variable, expected_value: f32, verbosity: i8) -> Result<bool, String> {
    let data = match var.values::<f32>(None, None) {
        Ok(arr) => arr,
        Err(err) => return Err(format!("Could not get data of '{}' variable: {}", var.name(), err))
    };

    let n_total = data.len();
    let mut n_wrong: usize = 0;

    for &value in data.iter() {
        // The ADCFs and AICFs are only written to 4 decimal places in the .aia file
        if !value.approx_eq(expected_value, F32Margin{ ulps: 1, epsilon: 1e-4}) {
            n_wrong += 1;
        }
    }

    
    let is_ok = _print_variable_results(&var.name(), n_total, n_wrong, verbosity);
    return Ok(is_ok)
}


// *************** //
// CHECK FUNCTIONS //
// *************** //

fn check_adcfs(nch: &netcdf::File, adcfs: &HashMap<&'static str, Adcf>, verbosity: i8) -> Result<bool, String> {
    // Get the windows in alphanumeric order
    let mut windows: Vec<&'static str> = adcfs.keys().map(|x| *x).collect();
    windows.sort_unstable();

    if verbosity > 1 {
        println!("=== Checking ADCF values ===");
    }

    let mut all_ok = true;
    for window in windows {
        let win_ok = check_one_adcf(nch, window, adcfs.get(window).unwrap(), verbosity)?;
        all_ok = all_ok && win_ok;
    }

    if verbosity == 1 {
        if all_ok {
            println!("* PASS: ADCFs match expected values");
        }else {
            println!("* FAIL: ADCFs do not match expected values");
        }
    }
    
    Ok(all_ok)
}

fn check_one_adcf(nch: &netcdf::File, window: &str, adcf: &Adcf, verbosity: i8) -> Result<bool, String> {
    if verbosity > 2 {
        println!("  * Checking {} ADCFS:", window);
    }

    let nc_adcfs = _get_var(nch, &format!("{}_adcf", window))?;
    let adcfs_ok = _all_equal_float(&nc_adcfs, adcf.adcf, verbosity)?;

    let nc_errs = _get_var(nch, &format!("{}_adcf_error", window))?;
    let errs_ok = _all_equal_float(&nc_errs, adcf.err, verbosity)?;

    let nc_g = _get_var(nch, &format!("{}_g", window))?;
    let g_ok = _all_equal_float(&nc_g, adcf.g as f32, verbosity)?;

    let nc_p = _get_var(nch, &format!("{}_p", window))?;
    let p_ok = _all_equal_float(&nc_p, adcf.p as f32, verbosity)?;

    let all_ok = adcfs_ok && errs_ok && g_ok && p_ok;

    if verbosity == 2 {
        if all_ok {
            println!("  - PASS: {} ADCFs are correct", window);
        }else{
            println!("  - FAIL: {} ADCFS are incorrect", window);
        }
    }

    Ok(all_ok)
}

fn check_aicfs(nch: &netcdf::File, aicfs: &HashMap<&'static str, Aicf>, verbosity: i8) -> Result<bool, String> {
    let mut gases: Vec<&'static str> = aicfs.keys().map(|x| *x).collect();
    gases.sort_unstable();

    if verbosity > 1 {
        println!("\n=== Checking AICF values ===");
    }

    let mut all_ok = true;
    for gas in gases {
        let gas_ok = check_one_aicf(nch, gas, aicfs.get(gas).unwrap(), verbosity)?;
        all_ok = all_ok && gas_ok;
    }

    if verbosity == 1 {
        if all_ok {
            println!("* PASS: AICFs match expected values");
        }else{
            println!("* FAIL: AICFs do not match expected values");
        }
    }

    Ok(all_ok)
}


fn check_one_aicf(nch: &netcdf::File, gas: &str, aicf: &Aicf, verbosity: i8) -> Result<bool, String> {
    let nc_aicfs = _get_var(nch, &format!("{}_aicf", gas))?;
    let aicfs_ok = _all_equal_float(&nc_aicfs, aicf.aicf, verbosity)?;

    let nc_errs = _get_var(nch, &format!("{}_aicf_error", gas))?;
    let errs_ok = _all_equal_float(&nc_errs, aicf.err, verbosity)?;

    let all_ok = aicfs_ok && errs_ok;

    if verbosity == 2 {
        if all_ok {
            println!("  - PASS: {} AICFS are correct", gas);
        }else{
            println!("  - FAIL: {} AICFS are not correct", gas);
        }
    }

    Ok(all_ok)
}


fn driver(nc_file: &str, verbosity: i8) -> Result<(), String> {
    
    let adcfs = read_adcf_table();
    let aicfs = read_aicf_table();
    let (windows, skipped_windows) = read_windows_table();

    let nch = match netcdf::open(nc_file) {
        Ok(h) => h,
        Err(err) => return Err(format!("Unable to open {}: {}", nc_file, err))
    };

    let adcfs_ok = check_adcfs(&nch, &adcfs, verbosity)?;
    let aicfs_ok = check_aicfs(&nch, &aicfs, verbosity)?;
    
    Ok(())
}

fn main() {
    let clargs: Vec<String> = env::args().collect();
    if clargs.len() != 2 || clargs[1] == "-h" || clargs[1] == "--help" {
        eprintln!("USAGE: {} PRIVATE_NC_FILE", clargs[0]);
        std::process::exit(1);
    }

    if let Err(msg) = driver(&clargs[1], 2) {
        eprintln!("ERROR: {}", msg);
        std::process::exit(2);
    }
}
