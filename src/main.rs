use std::collections::HashMap;
use std::env;
use clap;
use float_cmp::{ApproxEq,F32Margin};
use lazy_static::lazy_static;
use regex::Regex;

const GSETUP_VERSION: &'static str = "4.70; 2020-06-29; GCT";
const GFIT_VERSION: &'static str = "5.28; 2020-04-24; GCT";
const COLLATE_VERSION: &'static str = "2.09; 2020-07-31; GCT,JLL";
const AIRMASS_VERSION: &'static str = "1.38; 2020-12-16; GCT,JLL";
const AVERAGE_VERSION: &'static str = "1.37; 2020-07-31; GCT,JLL";
const INSITU_VERSION: &'static str = "1.39; 2020-07-31; GCT,JLL";
const WRITE_NC_HASH: &'static str = "42ed12d";

const ATT_MISSING_STR: &'static str = "!!MISSING!!";

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
\"xn2o\"   0.9821  0.0098  \"NOAA 2006A\"
\"xco\"    1.0000  0.0526  \"N/A\"
\"xh2o\"   0.9883  0.0157  \"ARM Radiosondes (Lamont+Darwin)\"
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

const EXPECTED_INGAAS_VARS: &'static str = "time,prior_time,cell_index,prior_altitude,ak_altitude,ak_slant_xgas_bin,ak_pressure,ak_slant_xco2_bin,ak_slant_xwco2_bin,ak_slant_xlco2_bin,ak_slant_xch4_bin,ak_slant_xhf_bin,ak_slant_xo2_bin,ak_slant_xn2o_bin,ak_slant_xco_bin,ak_slant_xh2o_bin,ak_xco2,ak_xwco2,ak_xlco2,ak_xch4,ak_xhf,ak_xo2,ak_xn2o,ak_xco,ak_xh2o,prior_index,prior_temperature,cell_temperature,prior_pressure,cell_pressure,prior_density,cell_density,prior_1h2o,cell_1h2o,prior_2h2o,cell_2h2o,prior_3h2o,cell_3h2o,prior_1co2,cell_1co2,prior_2co2,cell_2co2,prior_3co2,cell_3co2,prior_4co2,cell_4co2,prior_5co2,cell_5co2,prior_6co2,cell_6co2,prior_7co2,cell_7co2,prior_8co2,cell_8co2,prior_9co2,cell_9co2,prior_10co2,cell_10co2,prior_11co2,cell_11co2,prior_12co2,cell_12co2,prior_1o3,cell_1o3,prior_2o3,cell_2o3,prior_3o3,cell_3o3,prior_4o3,cell_4o3,prior_5o3,cell_5o3,prior_1n2o,cell_1n2o,prior_2n2o,cell_2n2o,prior_3n2o,cell_3n2o,prior_4n2o,cell_4n2o,prior_5n2o,cell_5n2o,prior_6n2o,cell_6n2o,prior_7n2o,cell_7n2o,prior_8n2o,cell_8n2o,prior_1co,cell_1co,prior_2co,cell_2co,prior_3co,cell_3co,prior_4co,cell_4co,prior_5co,cell_5co,prior_6co,cell_6co,prior_1ch4,cell_1ch4,prior_2ch4,cell_2ch4,prior_3ch4,cell_3ch4,prior_4ch4,cell_4ch4,prior_0o2,cell_0o2,prior_1o2,cell_1o2,prior_2o2,cell_2o2,prior_3o2,cell_3o2,prior_1no,cell_1no,prior_2no,cell_2no,prior_3no,cell_3no,prior_1so2,cell_1so2,prior_2so2,cell_2so2,prior_3so2,cell_3so2,prior_1no2,cell_1no2,prior_2no2,cell_2no2,prior_1nh3,cell_1nh3,prior_2nh3,cell_2nh3,prior_3nh3,cell_3nh3,prior_4nh3,cell_4nh3,prior_5nh3,cell_5nh3,prior_6nh3,cell_6nh3,prior_7nh3,cell_7nh3,prior_0hno3,cell_0hno3,prior_1hno3,cell_1hno3,prior_2hno3,cell_2hno3,prior_1oh,cell_1oh,prior_2oh,cell_2oh,prior_3oh,cell_3oh,prior_1hf,cell_1hf,prior_2hf,cell_2hf,prior_1hcl,cell_1hcl,prior_2hcl,cell_2hcl,prior_3hcl,cell_3hcl,prior_4hcl,cell_4hcl,prior_1hbr,cell_1hbr,prior_2hbr,cell_2hbr,prior_3hbr,cell_3hbr,prior_4hbr,cell_4hbr,prior_1hi,cell_1hi,prior_2hi,cell_2hi,prior_1clo,cell_1clo,prior_2clo,cell_2clo,prior_1ocs,cell_1ocs,prior_2ocs,cell_2ocs,prior_3ocs,cell_3ocs,prior_4ocs,cell_4ocs,prior_5ocs,cell_5ocs,prior_6ocs,cell_6ocs,prior_1h2co,cell_1h2co,prior_2h2co,cell_2h2co,prior_3h2co,cell_3h2co,prior_1hocl,cell_1hocl,prior_2hocl,cell_2hocl,prior_1ho2,cell_1ho2,prior_1h2o2,cell_1h2o2,prior_1hono,cell_1hono,prior_1ho2no2,cell_1ho2no2,prior_0n2o5,cell_0n2o5,prior_1n2o5,cell_1n2o5,prior_0clno3,cell_0clno3,prior_1clno3,cell_1clno3,prior_2clno3,cell_2clno3,prior_1hcn,cell_1hcn,prior_2hcn,cell_2hcn,prior_3hcn,cell_3hcn,prior_1ch3f,cell_1ch3f,prior_1ch3cl,cell_1ch3cl,prior_2ch3cl,cell_2ch3cl,prior_0cf4,cell_0cf4,prior_1cf4,cell_1cf4,prior_0ccl2f2,cell_0ccl2f2,prior_1ccl2f2,cell_1ccl2f2,prior_0ccl3f,cell_0ccl3f,prior_1ccl3f,cell_1ccl3f,prior_1ch3ccl3,cell_1ch3ccl3,prior_0ccl4,cell_0ccl4,prior_1ccl4,cell_1ccl4,prior_1cof2,cell_1cof2,prior_2cof2,cell_2cof2,prior_1cofcl,cell_1cofcl,prior_2cofcl,cell_2cofcl,prior_0c2h6,cell_0c2h6,prior_1c2h6,cell_1c2h6,prior_2c2h6,cell_2c2h6,prior_1c2h4,cell_1c2h4,prior_2c2h4,cell_2c2h4,prior_1c2h2,cell_1c2h2,prior_2c2h2,cell_2c2h2,prior_3c2h2,cell_3c2h2,prior_0n2,cell_0n2,prior_1n2,cell_1n2,prior_2n2,cell_2n2,prior_0chclf2,cell_0chclf2,prior_1chclf2,cell_1chclf2,prior_1cocl2,cell_1cocl2,prior_2cocl2,cell_2cocl2,prior_3cocl2,cell_3cocl2,prior_1ch3br,cell_1ch3br,prior_2ch3br,cell_2ch3br,prior_1ch3i,cell_1ch3i,prior_1hcooh,cell_1hcooh,prior_1h2s,cell_1h2s,prior_2h2s,cell_2h2s,prior_3h2s,cell_3h2s,prior_1chcl2f,cell_1chcl2f,prior_1hdo,cell_1hdo,prior_2hdo,cell_2hdo,prior_3hdo,cell_3hdo,prior_0sf6,cell_0sf6,prior_1sf6,cell_1sf6,prior_0f113,cell_0f113,prior_1f113,cell_1f113,prior_1clcn,cell_1clcn,prior_0f142b,cell_0f142b,prior_0dust_m,cell_0dust_m,prior_1ph3,cell_1ph3,prior_0ch3oh,cell_0ch3oh,prior_1ch3oh,cell_1ch3oh,prior_0ch3sh,cell_0ch3sh,prior_0ch3cho,cell_0ch3cho,prior_0ch3cn,cell_0ch3cn,prior_1ch3cn,cell_1ch3cn,prior_0pan,cell_0pan,prior_0nf3,cell_0nf3,prior_0cloocl,cell_0cloocl,prior_0clclo2,cell_0clclo2,prior_0cloclo,cell_0cloclo,prior_0chf3,cell_0chf3,prior_0f141b,cell_0f141b,prior_0ch3cooh,cell_0ch3cooh,prior_0cirrus6,cell_0cirrus6,prior_0cirrus15,cell_0cirrus15,prior_0c3h8,cell_0c3h8,prior_1c3h8,cell_1c3h8,prior_1d2o,cell_1d2o,prior_2d2o,cell_2d2o,prior_3d2o,cell_3d2o,prior_0sa_venus,cell_0sa_venus,prior_0c6h6,cell_0c6h6,prior_1c6h6,cell_1c6h6,prior_0c3h6,cell_0c3h6,prior_1c3h6,cell_1c3h6,prior_0ch3coch3,cell_0ch3coch3,prior_0cfh2cf3,cell_0cfh2cf3,prior_0n-c4h10,cell_0n-c4h10,prior_0c5h8,cell_0c5h8,prior_1luft,cell_1luft,prior_gravity,prior_equivalent_latitude,prior_tropopause_altitude,prior_modfile,prior_vmrfile,prior_effective_latitude,prior_mid_tropospheric_potential_temperature,config_checksum,apriori_checksum,runlog_checksum,levels_checksum,mav_checksum,ray_checksum,isotopologs_checksum,windows_checksum,telluric_linelists_checksum,solar_checksum,gfit_version,gsetup_version,flag,flagged_var_name,spectrum,year,day,hour,run,lat,long,zobs,zmin,solzen,azim,osds,opd,fovi,amal,graw,tins,pins,tout,pout,hout,sia,fvsi,wspd,wdir,tmod,pmod,h2o_dmf_out,h2o_dmf_mod,vsw_luft_6146,vsw_sf_luft_6146,vsw_ada_xluft_6146,vsw_luft_6146_error,vsw_ada_xluft_6146_error,vsw_hf_4038,vsw_sf_hf_4038,vsw_ada_xhf_4038,vsw_hf_4038_error,vsw_ada_xhf_4038_error,vsw_h2o_4565,vsw_sf_h2o_4565,vsw_ada_xh2o_4565,vsw_h2o_4565_error,vsw_ada_xh2o_4565_error,vsw_h2o_4570,vsw_sf_h2o_4570,vsw_ada_xh2o_4570,vsw_h2o_4570_error,vsw_ada_xh2o_4570_error,vsw_h2o_4571,vsw_sf_h2o_4571,vsw_ada_xh2o_4571,vsw_h2o_4571_error,vsw_ada_xh2o_4571_error,vsw_h2o_4576,vsw_sf_h2o_4576,vsw_ada_xh2o_4576,vsw_h2o_4576_error,vsw_ada_xh2o_4576_error,vsw_h2o_4598,vsw_sf_h2o_4598,vsw_ada_xh2o_4598,vsw_h2o_4598_error,vsw_ada_xh2o_4598_error,vsw_h2o_4611,vsw_sf_h2o_4611,vsw_ada_xh2o_4611,vsw_h2o_4611_error,vsw_ada_xh2o_4611_error,vsw_h2o_4622,vsw_sf_h2o_4622,vsw_ada_xh2o_4622,vsw_h2o_4622_error,vsw_ada_xh2o_4622_error,vsw_h2o_4631,vsw_sf_h2o_4631,vsw_ada_xh2o_4631,vsw_h2o_4631_error,vsw_ada_xh2o_4631_error,vsw_h2o_4699,vsw_sf_h2o_4699,vsw_ada_xh2o_4699,vsw_h2o_4699_error,vsw_ada_xh2o_4699_error,vsw_h2o_4734,vsw_sf_h2o_4734,vsw_ada_xh2o_4734,vsw_h2o_4734_error,vsw_ada_xh2o_4734_error,vsw_h2o_4761,vsw_sf_h2o_4761,vsw_ada_xh2o_4761,vsw_h2o_4761_error,vsw_ada_xh2o_4761_error,vsw_h2o_6076,vsw_sf_h2o_6076,vsw_ada_xh2o_6076,vsw_h2o_6076_error,vsw_ada_xh2o_6076_error,vsw_h2o_6099,vsw_sf_h2o_6099,vsw_ada_xh2o_6099,vsw_h2o_6099_error,vsw_ada_xh2o_6099_error,vsw_h2o_6125,vsw_sf_h2o_6125,vsw_ada_xh2o_6125,vsw_h2o_6125_error,vsw_ada_xh2o_6125_error,vsw_h2o_6177,vsw_sf_h2o_6177,vsw_ada_xh2o_6177,vsw_h2o_6177_error,vsw_ada_xh2o_6177_error,vsw_h2o_6255,vsw_sf_h2o_6255,vsw_ada_xh2o_6255,vsw_h2o_6255_error,vsw_ada_xh2o_6255_error,vsw_h2o_6301,vsw_sf_h2o_6301,vsw_ada_xh2o_6301,vsw_h2o_6301_error,vsw_ada_xh2o_6301_error,vsw_h2o_6392,vsw_sf_h2o_6392,vsw_ada_xh2o_6392,vsw_h2o_6392_error,vsw_ada_xh2o_6392_error,vsw_h2o_6401,vsw_sf_h2o_6401,vsw_ada_xh2o_6401,vsw_h2o_6401_error,vsw_ada_xh2o_6401_error,vsw_h2o_6469,vsw_sf_h2o_6469,vsw_ada_xh2o_6469,vsw_h2o_6469_error,vsw_ada_xh2o_6469_error,vsw_th2o_4054,vsw_sf_th2o_4054,vsw_ada_xth2o_4054,vsw_th2o_4054_error,vsw_ada_xth2o_4054_error,vsw_th2o_4255,vsw_sf_th2o_4255,vsw_ada_xth2o_4255,vsw_th2o_4255_error,vsw_ada_xth2o_4255_error,vsw_th2o_4325,vsw_sf_th2o_4325,vsw_ada_xth2o_4325,vsw_th2o_4325_error,vsw_ada_xth2o_4325_error,vsw_th2o_4493,vsw_sf_th2o_4493,vsw_ada_xth2o_4493,vsw_th2o_4493_error,vsw_ada_xth2o_4493_error,vsw_th2o_4516,vsw_sf_th2o_4516,vsw_ada_xth2o_4516,vsw_th2o_4516_error,vsw_ada_xth2o_4516_error,vsw_th2o_4524,vsw_sf_th2o_4524,vsw_ada_xth2o_4524,vsw_th2o_4524_error,vsw_ada_xth2o_4524_error,vsw_th2o_4633,vsw_sf_th2o_4633,vsw_ada_xth2o_4633,vsw_th2o_4633_error,vsw_ada_xth2o_4633_error,vsw_hdo_4054,vsw_sf_hdo_4054,vsw_ada_xhdo_4054,vsw_hdo_4054_error,vsw_ada_xhdo_4054_error,vsw_hdo_4067,vsw_sf_hdo_4067,vsw_ada_xhdo_4067,vsw_hdo_4067_error,vsw_ada_xhdo_4067_error,vsw_hdo_4116,vsw_sf_hdo_4116,vsw_ada_xhdo_4116,vsw_hdo_4116_error,vsw_ada_xhdo_4116_error,vsw_hdo_4212,vsw_sf_hdo_4212,vsw_ada_xhdo_4212,vsw_hdo_4212_error,vsw_ada_xhdo_4212_error,vsw_hdo_4232,vsw_sf_hdo_4232,vsw_ada_xhdo_4232,vsw_hdo_4232_error,vsw_ada_xhdo_4232_error,vsw_hdo_6330,vsw_sf_hdo_6330,vsw_ada_xhdo_6330,vsw_hdo_6330_error,vsw_ada_xhdo_6330_error,vsw_hdo_6377,vsw_sf_hdo_6377,vsw_ada_xhdo_6377,vsw_hdo_6377_error,vsw_ada_xhdo_6377_error,vsw_hdo_6458,vsw_sf_hdo_6458,vsw_ada_xhdo_6458,vsw_hdo_6458_error,vsw_ada_xhdo_6458_error,vsw_co_4290,vsw_sf_co_4290,vsw_ada_xco_4290,vsw_co_4290_error,vsw_ada_xco_4290_error,vsw_n2o_4395,vsw_sf_n2o_4395,vsw_ada_xn2o_4395,vsw_n2o_4395_error,vsw_ada_xn2o_4395_error,vsw_n2o_4430,vsw_sf_n2o_4430,vsw_ada_xn2o_4430,vsw_n2o_4430_error,vsw_ada_xn2o_4430_error,vsw_n2o_4719,vsw_sf_n2o_4719,vsw_ada_xn2o_4719,vsw_n2o_4719_error,vsw_ada_xn2o_4719_error,vsw_ch4_5938,vsw_sf_ch4_5938,vsw_ada_xch4_5938,vsw_ch4_5938_error,vsw_ada_xch4_5938_error,vsw_ch4_6002,vsw_sf_ch4_6002,vsw_ada_xch4_6002,vsw_ch4_6002_error,vsw_ada_xch4_6002_error,vsw_ch4_6076,vsw_sf_ch4_6076,vsw_ada_xch4_6076,vsw_ch4_6076_error,vsw_ada_xch4_6076_error,vsw_lco2_4852,vsw_sf_lco2_4852,vsw_ada_xlco2_4852,vsw_lco2_4852_error,vsw_ada_xlco2_4852_error,vsw_zco2_4852,vsw_sf_zco2_4852,vsw_ada_xzco2_4852,vsw_zco2_4852_error,vsw_ada_xzco2_4852_error,vsw_zco2_4852a,vsw_sf_zco2_4852a,vsw_ada_xzco2_4852a,vsw_zco2_4852a_error,vsw_ada_xzco2_4852a_error,vsw_fco2_6154,vsw_sf_fco2_6154,vsw_ada_xfco2_6154,vsw_fco2_6154_error,vsw_ada_xfco2_6154_error,vsw_wco2_6073,vsw_sf_wco2_6073,vsw_ada_xwco2_6073,vsw_wco2_6073_error,vsw_ada_xwco2_6073_error,vsw_co2_6220,vsw_sf_co2_6220,vsw_ada_xco2_6220,vsw_co2_6220_error,vsw_ada_xco2_6220_error,vsw_co2_6339,vsw_sf_co2_6339,vsw_ada_xco2_6339,vsw_co2_6339_error,vsw_ada_xco2_6339_error,vsw_o2_7885,vsw_sf_o2_7885,vsw_ada_xo2_7885,vsw_o2_7885_error,vsw_ada_xo2_7885_error,vsw_hcl_5625,vsw_sf_hcl_5625,vsw_ada_xhcl_5625,vsw_hcl_5625_error,vsw_ada_xhcl_5625_error,vsw_hcl_5687,vsw_sf_hcl_5687,vsw_ada_xhcl_5687,vsw_hcl_5687_error,vsw_ada_xhcl_5687_error,vsw_hcl_5702,vsw_sf_hcl_5702,vsw_ada_xhcl_5702,vsw_hcl_5702_error,vsw_ada_xhcl_5702_error,vsw_hcl_5735,vsw_sf_hcl_5735,vsw_ada_xhcl_5735,vsw_hcl_5735_error,vsw_ada_xhcl_5735_error,vsw_hcl_5739,vsw_sf_hcl_5739,vsw_ada_xhcl_5739,vsw_hcl_5739_error,vsw_ada_xhcl_5739_error,xluft,vsf_luft,column_luft,ada_xluft,xluft_error,vsf_luft_error,column_luft_error,ada_xluft_error,xhf,vsf_hf,column_hf,ada_xhf,xhf_error,vsf_hf_error,column_hf_error,ada_xhf_error,xh2o,vsf_h2o,column_h2o,ada_xh2o,xh2o_error,vsf_h2o_error,column_h2o_error,ada_xh2o_error,xth2o,vsf_th2o,column_th2o,ada_xth2o,xth2o_error,vsf_th2o_error,column_th2o_error,ada_xth2o_error,xhdo,vsf_hdo,column_hdo,ada_xhdo,xhdo_error,vsf_hdo_error,column_hdo_error,ada_xhdo_error,xco,vsf_co,column_co,ada_xco,xco_error,vsf_co_error,column_co_error,ada_xco_error,xn2o,vsf_n2o,column_n2o,ada_xn2o,xn2o_error,vsf_n2o_error,column_n2o_error,ada_xn2o_error,xch4,vsf_ch4,column_ch4,ada_xch4,xch4_error,vsf_ch4_error,column_ch4_error,ada_xch4_error,xlco2,vsf_lco2,column_lco2,ada_xlco2,xlco2_error,vsf_lco2_error,column_lco2_error,ada_xlco2_error,xzco2,vsf_zco2,column_zco2,ada_xzco2,xzco2_error,vsf_zco2_error,column_zco2_error,ada_xzco2_error,xfco2,vsf_fco2,column_fco2,ada_xfco2,xfco2_error,vsf_fco2_error,column_fco2_error,ada_xfco2_error,xwco2,vsf_wco2,column_wco2,ada_xwco2,xwco2_error,vsf_wco2_error,column_wco2_error,ada_xwco2_error,xco2,vsf_co2,column_co2,ada_xco2,xco2_error,vsf_co2_error,column_co2_error,ada_xco2_error,xo2,vsf_o2,column_o2,ada_xo2,xo2_error,vsf_o2_error,column_o2_error,ada_xo2_error,xhcl,vsf_hcl,column_hcl,ada_xhcl,xhcl_error,vsf_hcl_error,column_hcl_error,ada_xhcl_error,lst,lse,lsu,lsf,dip,mvd,xco2_6220_adcf,xco2_6220_adcf_error,xco2_6220_g,xco2_6220_p,xco2_6339_adcf,xco2_6339_adcf_error,xco2_6339_g,xco2_6339_p,xlco2_4852_adcf,xlco2_4852_adcf_error,xlco2_4852_g,xlco2_4852_p,xwco2_6073_adcf,xwco2_6073_adcf_error,xwco2_6073_g,xwco2_6073_p,xwco2_6500_adcf,xwco2_6500_adcf_error,xwco2_6500_g,xwco2_6500_p,xch4_5938_adcf,xch4_5938_adcf_error,xch4_5938_g,xch4_5938_p,xch4_6002_adcf,xch4_6002_adcf_error,xch4_6002_g,xch4_6002_p,xch4_6076_adcf,xch4_6076_adcf_error,xch4_6076_g,xch4_6076_p,xn2o_4395_adcf,xn2o_4395_adcf_error,xn2o_4395_g,xn2o_4395_p,xn2o_4430_adcf,xn2o_4430_adcf_error,xn2o_4430_g,xn2o_4430_p,xn2o_4719_adcf,xn2o_4719_adcf_error,xn2o_4719_g,xn2o_4719_p,xco_4233_adcf,xco_4233_adcf_error,xco_4233_g,xco_4233_p,xco_4290_adcf,xco_4290_adcf_error,xco_4290_g,xco_4290_p,xluft_6146_adcf,xluft_6146_adcf_error,xluft_6146_g,xluft_6146_p,xco2_aicf,xco2_aicf_error,aicf_xco2_scale,xwco2_aicf,xwco2_aicf_error,aicf_xwco2_scale,xlco2_aicf,xlco2_aicf_error,aicf_xlco2_scale,xch4_aicf,xch4_aicf_error,aicf_xch4_scale,xn2o_aicf,xn2o_aicf_error,aicf_xn2o_scale,xco_aicf,xco_aicf_error,aicf_xco_scale,xh2o_aicf,xh2o_aicf_error,aicf_xh2o_scale,xluft_aicf,xluft_aicf_error,aicf_xluft_scale,hf_4038_nit,hf_4038_cl,hf_4038_ct,hf_4038_cc,hf_4038_fs,hf_4038_sg,hf_4038_zo,hf_4038_rmsocl,hf_4038_zpres,hf_4038_am_hf,hf_4038_ovc_hf,hf_4038_vsf_hf,hf_4038_vsf_hf_error,hf_4038_am_h2o,hf_4038_ovc_h2o,hf_4038_vsf_h2o,hf_4038_vsf_h2o_error,hf_4038_ncbf,hf_4038_cfampocl,hf_4038_cfperiod,hf_4038_cfphase,hf_4038_cbf_01,hf_4038_cbf_02,h2o_4565_nit,h2o_4565_cl,h2o_4565_ct,h2o_4565_cc,h2o_4565_fs,h2o_4565_sg,h2o_4565_zo,h2o_4565_rmsocl,h2o_4565_zpres,h2o_4565_am_h2o,h2o_4565_ovc_h2o,h2o_4565_vsf_h2o,h2o_4565_vsf_h2o_error,h2o_4565_am_co2,h2o_4565_ovc_co2,h2o_4565_vsf_co2,h2o_4565_vsf_co2_error,h2o_4565_am_ch4,h2o_4565_ovc_ch4,h2o_4565_vsf_ch4,h2o_4565_vsf_ch4_error,h2o_4565_ncbf,h2o_4565_cfampocl,h2o_4565_cfperiod,h2o_4565_cfphase,h2o_4565_cbf_01,h2o_4565_cbf_02,h2o_4570_nit,h2o_4570_cl,h2o_4570_ct,h2o_4570_cc,h2o_4570_fs,h2o_4570_sg,h2o_4570_zo,h2o_4570_rmsocl,h2o_4570_zpres,h2o_4570_am_h2o,h2o_4570_ovc_h2o,h2o_4570_vsf_h2o,h2o_4570_vsf_h2o_error,h2o_4570_am_co2,h2o_4570_ovc_co2,h2o_4570_vsf_co2,h2o_4570_vsf_co2_error,h2o_4570_am_ch4,h2o_4570_ovc_ch4,h2o_4570_vsf_ch4,h2o_4570_vsf_ch4_error,h2o_4570_ncbf,h2o_4570_cfampocl,h2o_4570_cfperiod,h2o_4570_cfphase,h2o_4570_cbf_01,h2o_4570_cbf_02,h2o_4571_nit,h2o_4571_cl,h2o_4571_ct,h2o_4571_cc,h2o_4571_fs,h2o_4571_sg,h2o_4571_zo,h2o_4571_rmsocl,h2o_4571_zpres,h2o_4571_am_h2o,h2o_4571_ovc_h2o,h2o_4571_vsf_h2o,h2o_4571_vsf_h2o_error,h2o_4571_am_co2,h2o_4571_ovc_co2,h2o_4571_vsf_co2,h2o_4571_vsf_co2_error,h2o_4571_am_ch4,h2o_4571_ovc_ch4,h2o_4571_vsf_ch4,h2o_4571_vsf_ch4_error,h2o_4571_ncbf,h2o_4571_cfampocl,h2o_4571_cfperiod,h2o_4571_cfphase,h2o_4571_cbf_01,h2o_4571_cbf_02,h2o_4576_nit,h2o_4576_cl,h2o_4576_ct,h2o_4576_cc,h2o_4576_fs,h2o_4576_sg,h2o_4576_zo,h2o_4576_rmsocl,h2o_4576_zpres,h2o_4576_am_h2o,h2o_4576_ovc_h2o,h2o_4576_vsf_h2o,h2o_4576_vsf_h2o_error,h2o_4576_am_ch4,h2o_4576_ovc_ch4,h2o_4576_vsf_ch4,h2o_4576_vsf_ch4_error,h2o_4576_ncbf,h2o_4576_cfampocl,h2o_4576_cfperiod,h2o_4576_cfphase,h2o_4576_cbf_01,h2o_4576_cbf_02,h2o_4598_nit,h2o_4598_cl,h2o_4598_ct,h2o_4598_cc,h2o_4598_fs,h2o_4598_sg,h2o_4598_zo,h2o_4598_rmsocl,h2o_4598_zpres,h2o_4598_am_h2o,h2o_4598_ovc_h2o,h2o_4598_vsf_h2o,h2o_4598_vsf_h2o_error,h2o_4598_am_ch4,h2o_4598_ovc_ch4,h2o_4598_vsf_ch4,h2o_4598_vsf_ch4_error,h2o_4598_am_co2,h2o_4598_ovc_co2,h2o_4598_vsf_co2,h2o_4598_vsf_co2_error,h2o_4598_am_n2o,h2o_4598_ovc_n2o,h2o_4598_vsf_n2o,h2o_4598_vsf_n2o_error,h2o_4598_ncbf,h2o_4598_cfampocl,h2o_4598_cfperiod,h2o_4598_cfphase,h2o_4598_cbf_01,h2o_4598_cbf_02,h2o_4611_nit,h2o_4611_cl,h2o_4611_ct,h2o_4611_cc,h2o_4611_fs,h2o_4611_sg,h2o_4611_zo,h2o_4611_rmsocl,h2o_4611_zpres,h2o_4611_am_h2o,h2o_4611_ovc_h2o,h2o_4611_vsf_h2o,h2o_4611_vsf_h2o_error,h2o_4611_am_ch4,h2o_4611_ovc_ch4,h2o_4611_vsf_ch4,h2o_4611_vsf_ch4_error,h2o_4611_am_co2,h2o_4611_ovc_co2,h2o_4611_vsf_co2,h2o_4611_vsf_co2_error,h2o_4611_am_n2o,h2o_4611_ovc_n2o,h2o_4611_vsf_n2o,h2o_4611_vsf_n2o_error,h2o_4611_ncbf,h2o_4611_cfampocl,h2o_4611_cfperiod,h2o_4611_cfphase,h2o_4611_cbf_01,h2o_4611_cbf_02,h2o_4622_nit,h2o_4622_cl,h2o_4622_ct,h2o_4622_cc,h2o_4622_fs,h2o_4622_sg,h2o_4622_zo,h2o_4622_rmsocl,h2o_4622_zpres,h2o_4622_am_h2o,h2o_4622_ovc_h2o,h2o_4622_vsf_h2o,h2o_4622_vsf_h2o_error,h2o_4622_am_co2,h2o_4622_ovc_co2,h2o_4622_vsf_co2,h2o_4622_vsf_co2_error,h2o_4622_am_n2o,h2o_4622_ovc_n2o,h2o_4622_vsf_n2o,h2o_4622_vsf_n2o_error,h2o_4622_ncbf,h2o_4622_cfampocl,h2o_4622_cfperiod,h2o_4622_cfphase,h2o_4622_cbf_01,h2o_4622_cbf_02,h2o_4631_nit,h2o_4631_cl,h2o_4631_ct,h2o_4631_cc,h2o_4631_fs,h2o_4631_sg,h2o_4631_zo,h2o_4631_rmsocl,h2o_4631_zpres,h2o_4631_am_h2o,h2o_4631_ovc_h2o,h2o_4631_vsf_h2o,h2o_4631_vsf_h2o_error,h2o_4631_ncbf,h2o_4631_cfampocl,h2o_4631_cfperiod,h2o_4631_cfphase,h2o_4631_cbf_01,h2o_4631_cbf_02,h2o_4699_nit,h2o_4699_cl,h2o_4699_ct,h2o_4699_cc,h2o_4699_fs,h2o_4699_sg,h2o_4699_zo,h2o_4699_rmsocl,h2o_4699_zpres,h2o_4699_am_h2o,h2o_4699_ovc_h2o,h2o_4699_vsf_h2o,h2o_4699_vsf_h2o_error,h2o_4699_am_co2,h2o_4699_ovc_co2,h2o_4699_vsf_co2,h2o_4699_vsf_co2_error,h2o_4699_am_n2o,h2o_4699_ovc_n2o,h2o_4699_vsf_n2o,h2o_4699_vsf_n2o_error,h2o_4699_ncbf,h2o_4699_cfampocl,h2o_4699_cfperiod,h2o_4699_cfphase,h2o_4699_cbf_01,h2o_4699_cbf_02,h2o_4734_nit,h2o_4734_cl,h2o_4734_ct,h2o_4734_cc,h2o_4734_fs,h2o_4734_sg,h2o_4734_zo,h2o_4734_rmsocl,h2o_4734_zpres,h2o_4734_am_h2o,h2o_4734_ovc_h2o,h2o_4734_vsf_h2o,h2o_4734_vsf_h2o_error,h2o_4734_am_co2,h2o_4734_ovc_co2,h2o_4734_vsf_co2,h2o_4734_vsf_co2_error,h2o_4734_am_n2o,h2o_4734_ovc_n2o,h2o_4734_vsf_n2o,h2o_4734_vsf_n2o_error,h2o_4734_ncbf,h2o_4734_cfampocl,h2o_4734_cfperiod,h2o_4734_cfphase,h2o_4734_cbf_01,h2o_4734_cbf_02,h2o_4761_nit,h2o_4761_cl,h2o_4761_ct,h2o_4761_cc,h2o_4761_fs,h2o_4761_sg,h2o_4761_zo,h2o_4761_rmsocl,h2o_4761_zpres,h2o_4761_am_h2o,h2o_4761_ovc_h2o,h2o_4761_vsf_h2o,h2o_4761_vsf_h2o_error,h2o_4761_am_co2,h2o_4761_ovc_co2,h2o_4761_vsf_co2,h2o_4761_vsf_co2_error,h2o_4761_ncbf,h2o_4761_cfampocl,h2o_4761_cfperiod,h2o_4761_cfphase,h2o_4761_cbf_01,h2o_4761_cbf_02,h2o_6076_nit,h2o_6076_cl,h2o_6076_ct,h2o_6076_cc,h2o_6076_fs,h2o_6076_sg,h2o_6076_zo,h2o_6076_rmsocl,h2o_6076_zpres,h2o_6076_am_h2o,h2o_6076_ovc_h2o,h2o_6076_vsf_h2o,h2o_6076_vsf_h2o_error,h2o_6076_am_ch4,h2o_6076_ovc_ch4,h2o_6076_vsf_ch4,h2o_6076_vsf_ch4_error,h2o_6076_am_hdo,h2o_6076_ovc_hdo,h2o_6076_vsf_hdo,h2o_6076_vsf_hdo_error,h2o_6076_am_co2,h2o_6076_ovc_co2,h2o_6076_vsf_co2,h2o_6076_vsf_co2_error,h2o_6076_ncbf,h2o_6076_cfampocl,h2o_6076_cfperiod,h2o_6076_cfphase,h2o_6076_cbf_01,h2o_6076_cbf_02,h2o_6099_nit,h2o_6099_cl,h2o_6099_ct,h2o_6099_cc,h2o_6099_fs,h2o_6099_sg,h2o_6099_zo,h2o_6099_rmsocl,h2o_6099_zpres,h2o_6099_am_h2o,h2o_6099_ovc_h2o,h2o_6099_vsf_h2o,h2o_6099_vsf_h2o_error,h2o_6099_am_co2,h2o_6099_ovc_co2,h2o_6099_vsf_co2,h2o_6099_vsf_co2_error,h2o_6099_ncbf,h2o_6099_cfampocl,h2o_6099_cfperiod,h2o_6099_cfphase,h2o_6099_cbf_01,h2o_6099_cbf_02,h2o_6125_nit,h2o_6125_cl,h2o_6125_ct,h2o_6125_cc,h2o_6125_fs,h2o_6125_sg,h2o_6125_zo,h2o_6125_rmsocl,h2o_6125_zpres,h2o_6125_am_h2o,h2o_6125_ovc_h2o,h2o_6125_vsf_h2o,h2o_6125_vsf_h2o_error,h2o_6125_am_hdo,h2o_6125_ovc_hdo,h2o_6125_vsf_hdo,h2o_6125_vsf_hdo_error,h2o_6125_am_co2,h2o_6125_ovc_co2,h2o_6125_vsf_co2,h2o_6125_vsf_co2_error,h2o_6125_am_ch4,h2o_6125_ovc_ch4,h2o_6125_vsf_ch4,h2o_6125_vsf_ch4_error,h2o_6125_ncbf,h2o_6125_cfampocl,h2o_6125_cfperiod,h2o_6125_cfphase,h2o_6125_cbf_01,h2o_6125_cbf_02,h2o_6177_nit,h2o_6177_cl,h2o_6177_ct,h2o_6177_cc,h2o_6177_fs,h2o_6177_sg,h2o_6177_zo,h2o_6177_rmsocl,h2o_6177_zpres,h2o_6177_am_h2o,h2o_6177_ovc_h2o,h2o_6177_vsf_h2o,h2o_6177_vsf_h2o_error,h2o_6177_am_hdo,h2o_6177_ovc_hdo,h2o_6177_vsf_hdo,h2o_6177_vsf_hdo_error,h2o_6177_am_co2,h2o_6177_ovc_co2,h2o_6177_vsf_co2,h2o_6177_vsf_co2_error,h2o_6177_am_ch4,h2o_6177_ovc_ch4,h2o_6177_vsf_ch4,h2o_6177_vsf_ch4_error,h2o_6177_ncbf,h2o_6177_cfampocl,h2o_6177_cfperiod,h2o_6177_cfphase,h2o_6177_cbf_01,h2o_6177_cbf_02,h2o_6255_nit,h2o_6255_cl,h2o_6255_ct,h2o_6255_cc,h2o_6255_fs,h2o_6255_sg,h2o_6255_zo,h2o_6255_rmsocl,h2o_6255_zpres,h2o_6255_am_h2o,h2o_6255_ovc_h2o,h2o_6255_vsf_h2o,h2o_6255_vsf_h2o_error,h2o_6255_am_co2,h2o_6255_ovc_co2,h2o_6255_vsf_co2,h2o_6255_vsf_co2_error,h2o_6255_am_hdo,h2o_6255_ovc_hdo,h2o_6255_vsf_hdo,h2o_6255_vsf_hdo_error,h2o_6255_ncbf,h2o_6255_cfampocl,h2o_6255_cfperiod,h2o_6255_cfphase,h2o_6255_cbf_01,h2o_6255_cbf_02,h2o_6301_nit,h2o_6301_cl,h2o_6301_ct,h2o_6301_cc,h2o_6301_fs,h2o_6301_sg,h2o_6301_zo,h2o_6301_rmsocl,h2o_6301_zpres,h2o_6301_am_h2o,h2o_6301_ovc_h2o,h2o_6301_vsf_h2o,h2o_6301_vsf_h2o_error,h2o_6301_am_co2,h2o_6301_ovc_co2,h2o_6301_vsf_co2,h2o_6301_vsf_co2_error,h2o_6301_am_hdo,h2o_6301_ovc_hdo,h2o_6301_vsf_hdo,h2o_6301_vsf_hdo_error,h2o_6301_ncbf,h2o_6301_cfampocl,h2o_6301_cfperiod,h2o_6301_cfphase,h2o_6301_cbf_01,h2o_6301_cbf_02,h2o_6392_nit,h2o_6392_cl,h2o_6392_ct,h2o_6392_cc,h2o_6392_fs,h2o_6392_sg,h2o_6392_zo,h2o_6392_rmsocl,h2o_6392_zpres,h2o_6392_am_h2o,h2o_6392_ovc_h2o,h2o_6392_vsf_h2o,h2o_6392_vsf_h2o_error,h2o_6392_am_hdo,h2o_6392_ovc_hdo,h2o_6392_vsf_hdo,h2o_6392_vsf_hdo_error,h2o_6392_ncbf,h2o_6392_cfampocl,h2o_6392_cfperiod,h2o_6392_cfphase,h2o_6392_cbf_01,h2o_6392_cbf_02,h2o_6401_nit,h2o_6401_cl,h2o_6401_ct,h2o_6401_cc,h2o_6401_fs,h2o_6401_sg,h2o_6401_zo,h2o_6401_rmsocl,h2o_6401_zpres,h2o_6401_am_h2o,h2o_6401_ovc_h2o,h2o_6401_vsf_h2o,h2o_6401_vsf_h2o_error,h2o_6401_am_hdo,h2o_6401_ovc_hdo,h2o_6401_vsf_hdo,h2o_6401_vsf_hdo_error,h2o_6401_am_co2,h2o_6401_ovc_co2,h2o_6401_vsf_co2,h2o_6401_vsf_co2_error,h2o_6401_ncbf,h2o_6401_cfampocl,h2o_6401_cfperiod,h2o_6401_cfphase,h2o_6401_cbf_01,h2o_6401_cbf_02,h2o_6469_nit,h2o_6469_cl,h2o_6469_ct,h2o_6469_cc,h2o_6469_fs,h2o_6469_sg,h2o_6469_zo,h2o_6469_rmsocl,h2o_6469_zpres,h2o_6469_am_h2o,h2o_6469_ovc_h2o,h2o_6469_vsf_h2o,h2o_6469_vsf_h2o_error,h2o_6469_am_co2,h2o_6469_ovc_co2,h2o_6469_vsf_co2,h2o_6469_vsf_co2_error,h2o_6469_am_hdo,h2o_6469_ovc_hdo,h2o_6469_vsf_hdo,h2o_6469_vsf_hdo_error,h2o_6469_ncbf,h2o_6469_cfampocl,h2o_6469_cfperiod,h2o_6469_cfphase,h2o_6469_cbf_01,h2o_6469_cbf_02,th2o_4054_nit,th2o_4054_cl,th2o_4054_ct,th2o_4054_cc,th2o_4054_fs,th2o_4054_sg,th2o_4054_zo,th2o_4054_rmsocl,th2o_4054_zpres,th2o_4054_am_th2o,th2o_4054_ovc_th2o,th2o_4054_vsf_th2o,th2o_4054_vsf_th2o_error,th2o_4054_am_ch4,th2o_4054_ovc_ch4,th2o_4054_vsf_ch4,th2o_4054_vsf_ch4_error,th2o_4054_am_n2o,th2o_4054_ovc_n2o,th2o_4054_vsf_n2o,th2o_4054_vsf_n2o_error,th2o_4054_am_hdo,th2o_4054_ovc_hdo,th2o_4054_vsf_hdo,th2o_4054_vsf_hdo_error,th2o_4054_ncbf,th2o_4054_cfampocl,th2o_4054_cfperiod,th2o_4054_cfphase,th2o_4054_cbf_01,th2o_4054_cbf_02,th2o_4255_nit,th2o_4255_cl,th2o_4255_ct,th2o_4255_cc,th2o_4255_fs,th2o_4255_sg,th2o_4255_zo,th2o_4255_rmsocl,th2o_4255_zpres,th2o_4255_am_th2o,th2o_4255_ovc_th2o,th2o_4255_vsf_th2o,th2o_4255_vsf_th2o_error,th2o_4255_am_ch4,th2o_4255_ovc_ch4,th2o_4255_vsf_ch4,th2o_4255_vsf_ch4_error,th2o_4255_am_co,th2o_4255_ovc_co,th2o_4255_vsf_co,th2o_4255_vsf_co_error,th2o_4255_am_hdo,th2o_4255_ovc_hdo,th2o_4255_vsf_hdo,th2o_4255_vsf_hdo_error,th2o_4255_ncbf,th2o_4255_cfampocl,th2o_4255_cfperiod,th2o_4255_cfphase,th2o_4255_cbf_01,th2o_4255_cbf_02,th2o_4325_nit,th2o_4325_cl,th2o_4325_ct,th2o_4325_cc,th2o_4325_fs,th2o_4325_sg,th2o_4325_zo,th2o_4325_rmsocl,th2o_4325_zpres,th2o_4325_am_th2o,th2o_4325_ovc_th2o,th2o_4325_vsf_th2o,th2o_4325_vsf_th2o_error,th2o_4325_am_ch4,th2o_4325_ovc_ch4,th2o_4325_vsf_ch4,th2o_4325_vsf_ch4_error,th2o_4325_am_co,th2o_4325_ovc_co,th2o_4325_vsf_co,th2o_4325_vsf_co_error,th2o_4325_am_hdo,th2o_4325_ovc_hdo,th2o_4325_vsf_hdo,th2o_4325_vsf_hdo_error,th2o_4325_ncbf,th2o_4325_cfampocl,th2o_4325_cfperiod,th2o_4325_cfphase,th2o_4325_cbf_01,th2o_4325_cbf_02,th2o_4493_nit,th2o_4493_cl,th2o_4493_ct,th2o_4493_cc,th2o_4493_fs,th2o_4493_sg,th2o_4493_zo,th2o_4493_rmsocl,th2o_4493_zpres,th2o_4493_am_th2o,th2o_4493_ovc_th2o,th2o_4493_vsf_th2o,th2o_4493_vsf_th2o_error,th2o_4493_am_ch4,th2o_4493_ovc_ch4,th2o_4493_vsf_ch4,th2o_4493_vsf_ch4_error,th2o_4493_ncbf,th2o_4493_cfampocl,th2o_4493_cfperiod,th2o_4493_cfphase,th2o_4493_cbf_01,th2o_4493_cbf_02,th2o_4516_nit,th2o_4516_cl,th2o_4516_ct,th2o_4516_cc,th2o_4516_fs,th2o_4516_sg,th2o_4516_zo,th2o_4516_rmsocl,th2o_4516_zpres,th2o_4516_am_th2o,th2o_4516_ovc_th2o,th2o_4516_vsf_th2o,th2o_4516_vsf_th2o_error,th2o_4516_am_ch4,th2o_4516_ovc_ch4,th2o_4516_vsf_ch4,th2o_4516_vsf_ch4_error,th2o_4516_ncbf,th2o_4516_cfampocl,th2o_4516_cfperiod,th2o_4516_cfphase,th2o_4516_cbf_01,th2o_4516_cbf_02,th2o_4524_nit,th2o_4524_cl,th2o_4524_ct,th2o_4524_cc,th2o_4524_fs,th2o_4524_sg,th2o_4524_zo,th2o_4524_rmsocl,th2o_4524_zpres,th2o_4524_am_th2o,th2o_4524_ovc_th2o,th2o_4524_vsf_th2o,th2o_4524_vsf_th2o_error,th2o_4524_am_ch4,th2o_4524_ovc_ch4,th2o_4524_vsf_ch4,th2o_4524_vsf_ch4_error,th2o_4524_am_co2,th2o_4524_ovc_co2,th2o_4524_vsf_co2,th2o_4524_vsf_co2_error,th2o_4524_ncbf,th2o_4524_cfampocl,th2o_4524_cfperiod,th2o_4524_cfphase,th2o_4524_cbf_01,th2o_4524_cbf_02,th2o_4633_nit,th2o_4633_cl,th2o_4633_ct,th2o_4633_cc,th2o_4633_fs,th2o_4633_sg,th2o_4633_zo,th2o_4633_rmsocl,th2o_4633_zpres,th2o_4633_am_th2o,th2o_4633_ovc_th2o,th2o_4633_vsf_th2o,th2o_4633_vsf_th2o_error,th2o_4633_am_co2,th2o_4633_ovc_co2,th2o_4633_vsf_co2,th2o_4633_vsf_co2_error,th2o_4633_am_n2o,th2o_4633_ovc_n2o,th2o_4633_vsf_n2o,th2o_4633_vsf_n2o_error,th2o_4633_ncbf,th2o_4633_cfampocl,th2o_4633_cfperiod,th2o_4633_cfphase,th2o_4633_cbf_01,th2o_4633_cbf_02,hdo_4054_nit,hdo_4054_cl,hdo_4054_ct,hdo_4054_cc,hdo_4054_fs,hdo_4054_sg,hdo_4054_zo,hdo_4054_rmsocl,hdo_4054_zpres,hdo_4054_am_hdo,hdo_4054_ovc_hdo,hdo_4054_vsf_hdo,hdo_4054_vsf_hdo_error,hdo_4054_am_h2o,hdo_4054_ovc_h2o,hdo_4054_vsf_h2o,hdo_4054_vsf_h2o_error,hdo_4054_am_ch4,hdo_4054_ovc_ch4,hdo_4054_vsf_ch4,hdo_4054_vsf_ch4_error,hdo_4054_ncbf,hdo_4054_cfampocl,hdo_4054_cfperiod,hdo_4054_cfphase,hdo_4054_cbf_01,hdo_4054_cbf_02,hdo_4067_nit,hdo_4067_cl,hdo_4067_ct,hdo_4067_cc,hdo_4067_fs,hdo_4067_sg,hdo_4067_zo,hdo_4067_rmsocl,hdo_4067_zpres,hdo_4067_am_hdo,hdo_4067_ovc_hdo,hdo_4067_vsf_hdo,hdo_4067_vsf_hdo_error,hdo_4067_am_h2o,hdo_4067_ovc_h2o,hdo_4067_vsf_h2o,hdo_4067_vsf_h2o_error,hdo_4067_am_ch4,hdo_4067_ovc_ch4,hdo_4067_vsf_ch4,hdo_4067_vsf_ch4_error,hdo_4067_ncbf,hdo_4067_cfampocl,hdo_4067_cfperiod,hdo_4067_cfphase,hdo_4067_cbf_01,hdo_4067_cbf_02,hdo_4116_nit,hdo_4116_cl,hdo_4116_ct,hdo_4116_cc,hdo_4116_fs,hdo_4116_sg,hdo_4116_zo,hdo_4116_rmsocl,hdo_4116_zpres,hdo_4116_am_hdo,hdo_4116_ovc_hdo,hdo_4116_vsf_hdo,hdo_4116_vsf_hdo_error,hdo_4116_am_h2o,hdo_4116_ovc_h2o,hdo_4116_vsf_h2o,hdo_4116_vsf_h2o_error,hdo_4116_am_ch4,hdo_4116_ovc_ch4,hdo_4116_vsf_ch4,hdo_4116_vsf_ch4_error,hdo_4116_ncbf,hdo_4116_cfampocl,hdo_4116_cfperiod,hdo_4116_cfphase,hdo_4116_cbf_01,hdo_4116_cbf_02,hdo_4212_nit,hdo_4212_cl,hdo_4212_ct,hdo_4212_cc,hdo_4212_fs,hdo_4212_sg,hdo_4212_zo,hdo_4212_rmsocl,hdo_4212_zpres,hdo_4212_am_hdo,hdo_4212_ovc_hdo,hdo_4212_vsf_hdo,hdo_4212_vsf_hdo_error,hdo_4212_am_h2o,hdo_4212_ovc_h2o,hdo_4212_vsf_h2o,hdo_4212_vsf_h2o_error,hdo_4212_am_ch4,hdo_4212_ovc_ch4,hdo_4212_vsf_ch4,hdo_4212_vsf_ch4_error,hdo_4212_ncbf,hdo_4212_cfampocl,hdo_4212_cfperiod,hdo_4212_cfphase,hdo_4212_cbf_01,hdo_4212_cbf_02,hdo_4232_nit,hdo_4232_cl,hdo_4232_ct,hdo_4232_cc,hdo_4232_fs,hdo_4232_sg,hdo_4232_zo,hdo_4232_rmsocl,hdo_4232_zpres,hdo_4232_am_hdo,hdo_4232_ovc_hdo,hdo_4232_vsf_hdo,hdo_4232_vsf_hdo_error,hdo_4232_am_h2o,hdo_4232_ovc_h2o,hdo_4232_vsf_h2o,hdo_4232_vsf_h2o_error,hdo_4232_am_ch4,hdo_4232_ovc_ch4,hdo_4232_vsf_ch4,hdo_4232_vsf_ch4_error,hdo_4232_am_co,hdo_4232_ovc_co,hdo_4232_vsf_co,hdo_4232_vsf_co_error,hdo_4232_ncbf,hdo_4232_cfampocl,hdo_4232_cfperiod,hdo_4232_cfphase,hdo_4232_cbf_01,hdo_4232_cbf_02,hdo_6330_nit,hdo_6330_cl,hdo_6330_ct,hdo_6330_cc,hdo_6330_fs,hdo_6330_sg,hdo_6330_zo,hdo_6330_rmsocl,hdo_6330_zpres,hdo_6330_am_hdo,hdo_6330_ovc_hdo,hdo_6330_vsf_hdo,hdo_6330_vsf_hdo_error,hdo_6330_am_h2o,hdo_6330_ovc_h2o,hdo_6330_vsf_h2o,hdo_6330_vsf_h2o_error,hdo_6330_am_co2,hdo_6330_ovc_co2,hdo_6330_vsf_co2,hdo_6330_vsf_co2_error,hdo_6330_ncbf,hdo_6330_cfampocl,hdo_6330_cfperiod,hdo_6330_cfphase,hdo_6330_cbf_01,hdo_6330_cbf_02,hdo_6330_cbf_03,hdo_6330_cbf_04,hdo_6377_nit,hdo_6377_cl,hdo_6377_ct,hdo_6377_cc,hdo_6377_fs,hdo_6377_sg,hdo_6377_zo,hdo_6377_rmsocl,hdo_6377_zpres,hdo_6377_am_hdo,hdo_6377_ovc_hdo,hdo_6377_vsf_hdo,hdo_6377_vsf_hdo_error,hdo_6377_am_h2o,hdo_6377_ovc_h2o,hdo_6377_vsf_h2o,hdo_6377_vsf_h2o_error,hdo_6377_am_co2,hdo_6377_ovc_co2,hdo_6377_vsf_co2,hdo_6377_vsf_co2_error,hdo_6377_ncbf,hdo_6377_cfampocl,hdo_6377_cfperiod,hdo_6377_cfphase,hdo_6377_cbf_01,hdo_6377_cbf_02,hdo_6377_cbf_03,hdo_6377_cbf_04,hdo_6458_nit,hdo_6458_cl,hdo_6458_ct,hdo_6458_cc,hdo_6458_fs,hdo_6458_sg,hdo_6458_zo,hdo_6458_rmsocl,hdo_6458_zpres,hdo_6458_am_hdo,hdo_6458_ovc_hdo,hdo_6458_vsf_hdo,hdo_6458_vsf_hdo_error,hdo_6458_am_h2o,hdo_6458_ovc_h2o,hdo_6458_vsf_h2o,hdo_6458_vsf_h2o_error,hdo_6458_am_co2,hdo_6458_ovc_co2,hdo_6458_vsf_co2,hdo_6458_vsf_co2_error,hdo_6458_ncbf,hdo_6458_cfampocl,hdo_6458_cfperiod,hdo_6458_cfphase,hdo_6458_cbf_01,hdo_6458_cbf_02,hdo_6458_cbf_03,hdo_6458_cbf_04,co_4290_nit,co_4290_cl,co_4290_ct,co_4290_cc,co_4290_fs,co_4290_sg,co_4290_zo,co_4290_rmsocl,co_4290_zpres,co_4290_am_co,co_4290_ovc_co,co_4290_vsf_co,co_4290_vsf_co_error,co_4290_am_ch4,co_4290_ovc_ch4,co_4290_vsf_ch4,co_4290_vsf_ch4_error,co_4290_am_h2o,co_4290_ovc_h2o,co_4290_vsf_h2o,co_4290_vsf_h2o_error,co_4290_am_hdo,co_4290_ovc_hdo,co_4290_vsf_hdo,co_4290_vsf_hdo_error,co_4290_ncbf,co_4290_cfampocl,co_4290_cfperiod,co_4290_cfphase,co_4290_cbf_01,co_4290_cbf_02,co_4290_cbf_03,co_4290_cbf_04,n2o_4395_nit,n2o_4395_cl,n2o_4395_ct,n2o_4395_cc,n2o_4395_fs,n2o_4395_sg,n2o_4395_zo,n2o_4395_rmsocl,n2o_4395_zpres,n2o_4395_am_n2o,n2o_4395_ovc_n2o,n2o_4395_vsf_n2o,n2o_4395_vsf_n2o_error,n2o_4395_am_ch4,n2o_4395_ovc_ch4,n2o_4395_vsf_ch4,n2o_4395_vsf_ch4_error,n2o_4395_am_h2o,n2o_4395_ovc_h2o,n2o_4395_vsf_h2o,n2o_4395_vsf_h2o_error,n2o_4395_am_hdo,n2o_4395_ovc_hdo,n2o_4395_vsf_hdo,n2o_4395_vsf_hdo_error,n2o_4395_ncbf,n2o_4395_cfampocl,n2o_4395_cfperiod,n2o_4395_cfphase,n2o_4395_cbf_01,n2o_4395_cbf_02,n2o_4395_cbf_03,n2o_4395_cbf_04,n2o_4430_nit,n2o_4430_cl,n2o_4430_ct,n2o_4430_cc,n2o_4430_fs,n2o_4430_sg,n2o_4430_zo,n2o_4430_rmsocl,n2o_4430_zpres,n2o_4430_am_n2o,n2o_4430_ovc_n2o,n2o_4430_vsf_n2o,n2o_4430_vsf_n2o_error,n2o_4430_am_ch4,n2o_4430_ovc_ch4,n2o_4430_vsf_ch4,n2o_4430_vsf_ch4_error,n2o_4430_am_h2o,n2o_4430_ovc_h2o,n2o_4430_vsf_h2o,n2o_4430_vsf_h2o_error,n2o_4430_am_hdo,n2o_4430_ovc_hdo,n2o_4430_vsf_hdo,n2o_4430_vsf_hdo_error,n2o_4430_am_co2,n2o_4430_ovc_co2,n2o_4430_vsf_co2,n2o_4430_vsf_co2_error,n2o_4430_ncbf,n2o_4430_cfampocl,n2o_4430_cfperiod,n2o_4430_cfphase,n2o_4430_cbf_01,n2o_4430_cbf_02,n2o_4719_nit,n2o_4719_cl,n2o_4719_ct,n2o_4719_cc,n2o_4719_fs,n2o_4719_sg,n2o_4719_zo,n2o_4719_rmsocl,n2o_4719_zpres,n2o_4719_am_n2o,n2o_4719_ovc_n2o,n2o_4719_vsf_n2o,n2o_4719_vsf_n2o_error,n2o_4719_am_ch4,n2o_4719_ovc_ch4,n2o_4719_vsf_ch4,n2o_4719_vsf_ch4_error,n2o_4719_am_h2o,n2o_4719_ovc_h2o,n2o_4719_vsf_h2o,n2o_4719_vsf_h2o_error,n2o_4719_am_co2,n2o_4719_ovc_co2,n2o_4719_vsf_co2,n2o_4719_vsf_co2_error,n2o_4719_ncbf,n2o_4719_cfampocl,n2o_4719_cfperiod,n2o_4719_cfphase,n2o_4719_cbf_01,n2o_4719_cbf_02,n2o_4719_cbf_03,ch4_5938_nit,ch4_5938_cl,ch4_5938_ct,ch4_5938_cc,ch4_5938_fs,ch4_5938_sg,ch4_5938_zo,ch4_5938_rmsocl,ch4_5938_zpres,ch4_5938_am_ch4,ch4_5938_ovc_ch4,ch4_5938_vsf_ch4,ch4_5938_vsf_ch4_error,ch4_5938_am_co2,ch4_5938_ovc_co2,ch4_5938_vsf_co2,ch4_5938_vsf_co2_error,ch4_5938_am_h2o,ch4_5938_ovc_h2o,ch4_5938_vsf_h2o,ch4_5938_vsf_h2o_error,ch4_5938_am_n2o,ch4_5938_ovc_n2o,ch4_5938_vsf_n2o,ch4_5938_vsf_n2o_error,ch4_5938_ncbf,ch4_5938_cfampocl,ch4_5938_cfperiod,ch4_5938_cfphase,ch4_5938_cbf_01,ch4_5938_cbf_02,ch4_5938_cbf_03,ch4_5938_cbf_04,ch4_6002_nit,ch4_6002_cl,ch4_6002_ct,ch4_6002_cc,ch4_6002_fs,ch4_6002_sg,ch4_6002_zo,ch4_6002_rmsocl,ch4_6002_zpres,ch4_6002_am_ch4,ch4_6002_ovc_ch4,ch4_6002_vsf_ch4,ch4_6002_vsf_ch4_error,ch4_6002_am_co2,ch4_6002_ovc_co2,ch4_6002_vsf_co2,ch4_6002_vsf_co2_error,ch4_6002_am_h2o,ch4_6002_ovc_h2o,ch4_6002_vsf_h2o,ch4_6002_vsf_h2o_error,ch4_6002_am_hdo,ch4_6002_ovc_hdo,ch4_6002_vsf_hdo,ch4_6002_vsf_hdo_error,ch4_6002_ncbf,ch4_6002_cfampocl,ch4_6002_cfperiod,ch4_6002_cfphase,ch4_6002_cbf_01,ch4_6002_cbf_02,ch4_6076_nit,ch4_6076_cl,ch4_6076_ct,ch4_6076_cc,ch4_6076_fs,ch4_6076_sg,ch4_6076_zo,ch4_6076_rmsocl,ch4_6076_zpres,ch4_6076_am_ch4,ch4_6076_ovc_ch4,ch4_6076_vsf_ch4,ch4_6076_vsf_ch4_error,ch4_6076_am_co2,ch4_6076_ovc_co2,ch4_6076_vsf_co2,ch4_6076_vsf_co2_error,ch4_6076_am_h2o,ch4_6076_ovc_h2o,ch4_6076_vsf_h2o,ch4_6076_vsf_h2o_error,ch4_6076_am_hdo,ch4_6076_ovc_hdo,ch4_6076_vsf_hdo,ch4_6076_vsf_hdo_error,ch4_6076_ncbf,ch4_6076_cfampocl,ch4_6076_cfperiod,ch4_6076_cfphase,ch4_6076_cbf_01,ch4_6076_cbf_02,ch4_6076_cbf_03,ch4_6076_cbf_04,ch4_6076_cbf_05,lco2_4852_nit,lco2_4852_cl,lco2_4852_ct,lco2_4852_cc,lco2_4852_fs,lco2_4852_sg,lco2_4852_zo,lco2_4852_rmsocl,lco2_4852_zpres,lco2_4852_am_lco2,lco2_4852_ovc_lco2,lco2_4852_vsf_lco2,lco2_4852_vsf_lco2_error,lco2_4852_am_2co2,lco2_4852_ovc_2co2,lco2_4852_vsf_2co2,lco2_4852_vsf_2co2_error,lco2_4852_am_3co2,lco2_4852_ovc_3co2,lco2_4852_vsf_3co2,lco2_4852_vsf_3co2_error,lco2_4852_am_4co2,lco2_4852_ovc_4co2,lco2_4852_vsf_4co2,lco2_4852_vsf_4co2_error,lco2_4852_am_h2o,lco2_4852_ovc_h2o,lco2_4852_vsf_h2o,lco2_4852_vsf_h2o_error,lco2_4852_am_hdo,lco2_4852_ovc_hdo,lco2_4852_vsf_hdo,lco2_4852_vsf_hdo_error,lco2_4852_ncbf,lco2_4852_cfampocl,lco2_4852_cfperiod,lco2_4852_cfphase,lco2_4852_cbf_01,lco2_4852_cbf_02,lco2_4852_cbf_03,zco2_4852_nit,zco2_4852_cl,zco2_4852_ct,zco2_4852_cc,zco2_4852_fs,zco2_4852_sg,zco2_4852_zo,zco2_4852_rmsocl,zco2_4852_zpres,zco2_4852_am_zco2,zco2_4852_ovc_zco2,zco2_4852_vsf_zco2,zco2_4852_vsf_zco2_error,zco2_4852_am_h2o,zco2_4852_ovc_h2o,zco2_4852_vsf_h2o,zco2_4852_vsf_h2o_error,zco2_4852_am_hdo,zco2_4852_ovc_hdo,zco2_4852_vsf_hdo,zco2_4852_vsf_hdo_error,zco2_4852_ncbf,zco2_4852_cfampocl,zco2_4852_cfperiod,zco2_4852_cfphase,zco2_4852_cbf_01,zco2_4852_cbf_02,zco2_4852_cbf_03,zco2_4852a_nit,zco2_4852a_cl,zco2_4852a_ct,zco2_4852a_cc,zco2_4852a_fs,zco2_4852a_sg,zco2_4852a_zo,zco2_4852a_rmsocl,zco2_4852a_zpres,zco2_4852a_am_zco2,zco2_4852a_ovc_zco2,zco2_4852a_vsf_zco2,zco2_4852a_vsf_zco2_error,zco2_4852a_am_h2o,zco2_4852a_ovc_h2o,zco2_4852a_vsf_h2o,zco2_4852a_vsf_h2o_error,zco2_4852a_am_hdo,zco2_4852a_ovc_hdo,zco2_4852a_vsf_hdo,zco2_4852a_vsf_hdo_error,zco2_4852a_ncbf,zco2_4852a_cfampocl,zco2_4852a_cfperiod,zco2_4852a_cfphase,zco2_4852a_cbf_01,zco2_4852a_cbf_02,zco2_4852a_cbf_03,fco2_6154_nit,fco2_6154_cl,fco2_6154_ct,fco2_6154_cc,fco2_6154_fs,fco2_6154_sg,fco2_6154_zo,fco2_6154_rmsocl,fco2_6154_zpres,fco2_6154_am_fco2,fco2_6154_ovc_fco2,fco2_6154_vsf_fco2,fco2_6154_vsf_fco2_error,fco2_6154_am_h2o,fco2_6154_ovc_h2o,fco2_6154_vsf_h2o,fco2_6154_vsf_h2o_error,fco2_6154_am_hdo,fco2_6154_ovc_hdo,fco2_6154_vsf_hdo,fco2_6154_vsf_hdo_error,fco2_6154_am_ch4,fco2_6154_ovc_ch4,fco2_6154_vsf_ch4,fco2_6154_vsf_ch4_error,fco2_6154_ncbf,fco2_6154_cfampocl,fco2_6154_cfperiod,fco2_6154_cfphase,fco2_6154_cbf_01,fco2_6154_cbf_02,fco2_6154_cbf_03,fco2_6154_cbf_04,wco2_6073_nit,wco2_6073_cl,wco2_6073_ct,wco2_6073_cc,wco2_6073_fs,wco2_6073_sg,wco2_6073_zo,wco2_6073_rmsocl,wco2_6073_zpres,wco2_6073_am_wco2,wco2_6073_ovc_wco2,wco2_6073_vsf_wco2,wco2_6073_vsf_wco2_error,wco2_6073_am_h2o,wco2_6073_ovc_h2o,wco2_6073_vsf_h2o,wco2_6073_vsf_h2o_error,wco2_6073_am_ch4,wco2_6073_ovc_ch4,wco2_6073_vsf_ch4,wco2_6073_vsf_ch4_error,wco2_6073_ncbf,wco2_6073_cfampocl,wco2_6073_cfperiod,wco2_6073_cfphase,wco2_6073_cbf_01,wco2_6073_cbf_02,co2_6220_nit,co2_6220_cl,co2_6220_ct,co2_6220_cc,co2_6220_fs,co2_6220_sg,co2_6220_zo,co2_6220_rmsocl,co2_6220_zpres,co2_6220_am_co2,co2_6220_ovc_co2,co2_6220_vsf_co2,co2_6220_vsf_co2_error,co2_6220_am_h2o,co2_6220_ovc_h2o,co2_6220_vsf_h2o,co2_6220_vsf_h2o_error,co2_6220_am_hdo,co2_6220_ovc_hdo,co2_6220_vsf_hdo,co2_6220_vsf_hdo_error,co2_6220_am_ch4,co2_6220_ovc_ch4,co2_6220_vsf_ch4,co2_6220_vsf_ch4_error,co2_6220_ncbf,co2_6220_cfampocl,co2_6220_cfperiod,co2_6220_cfphase,co2_6220_cbf_01,co2_6220_cbf_02,co2_6220_cbf_03,co2_6339_nit,co2_6339_cl,co2_6339_ct,co2_6339_cc,co2_6339_fs,co2_6339_sg,co2_6339_zo,co2_6339_rmsocl,co2_6339_zpres,co2_6339_am_co2,co2_6339_ovc_co2,co2_6339_vsf_co2,co2_6339_vsf_co2_error,co2_6339_am_h2o,co2_6339_ovc_h2o,co2_6339_vsf_h2o,co2_6339_vsf_h2o_error,co2_6339_am_hdo,co2_6339_ovc_hdo,co2_6339_vsf_hdo,co2_6339_vsf_hdo_error,co2_6339_ncbf,co2_6339_cfampocl,co2_6339_cfperiod,co2_6339_cfphase,co2_6339_cbf_01,co2_6339_cbf_02,co2_6339_cbf_03,o2_7885_nit,o2_7885_cl,o2_7885_ct,o2_7885_cc,o2_7885_fs,o2_7885_sg,o2_7885_zo,o2_7885_rmsocl,o2_7885_zpres,o2_7885_am_o2,o2_7885_ovc_o2,o2_7885_vsf_o2,o2_7885_vsf_o2_error,o2_7885_am_0o2,o2_7885_ovc_0o2,o2_7885_vsf_0o2,o2_7885_vsf_0o2_error,o2_7885_am_h2o,o2_7885_ovc_h2o,o2_7885_vsf_h2o,o2_7885_vsf_h2o_error,o2_7885_am_hf,o2_7885_ovc_hf,o2_7885_vsf_hf,o2_7885_vsf_hf_error,o2_7885_am_co2,o2_7885_ovc_co2,o2_7885_vsf_co2,o2_7885_vsf_co2_error,o2_7885_am_hdo,o2_7885_ovc_hdo,o2_7885_vsf_hdo,o2_7885_vsf_hdo_error,o2_7885_ncbf,o2_7885_cfampocl,o2_7885_cfperiod,o2_7885_cfphase,o2_7885_cbf_01,o2_7885_cbf_02,o2_7885_cbf_03,o2_7885_cbf_04,o2_7885_cbf_05,hcl_5625_nit,hcl_5625_cl,hcl_5625_ct,hcl_5625_cc,hcl_5625_fs,hcl_5625_sg,hcl_5625_zo,hcl_5625_rmsocl,hcl_5625_zpres,hcl_5625_am_hcl,hcl_5625_ovc_hcl,hcl_5625_vsf_hcl,hcl_5625_vsf_hcl_error,hcl_5625_am_h2o,hcl_5625_ovc_h2o,hcl_5625_vsf_h2o,hcl_5625_vsf_h2o_error,hcl_5625_am_ch4,hcl_5625_ovc_ch4,hcl_5625_vsf_ch4,hcl_5625_vsf_ch4_error,hcl_5625_ncbf,hcl_5625_cfampocl,hcl_5625_cfperiod,hcl_5625_cfphase,hcl_5625_cbf_01,hcl_5625_cbf_02,hcl_5687_nit,hcl_5687_cl,hcl_5687_ct,hcl_5687_cc,hcl_5687_fs,hcl_5687_sg,hcl_5687_zo,hcl_5687_rmsocl,hcl_5687_zpres,hcl_5687_am_hcl,hcl_5687_ovc_hcl,hcl_5687_vsf_hcl,hcl_5687_vsf_hcl_error,hcl_5687_am_h2o,hcl_5687_ovc_h2o,hcl_5687_vsf_h2o,hcl_5687_vsf_h2o_error,hcl_5687_am_ch4,hcl_5687_ovc_ch4,hcl_5687_vsf_ch4,hcl_5687_vsf_ch4_error,hcl_5687_ncbf,hcl_5687_cfampocl,hcl_5687_cfperiod,hcl_5687_cfphase,hcl_5687_cbf_01,hcl_5687_cbf_02,hcl_5702_nit,hcl_5702_cl,hcl_5702_ct,hcl_5702_cc,hcl_5702_fs,hcl_5702_sg,hcl_5702_zo,hcl_5702_rmsocl,hcl_5702_zpres,hcl_5702_am_hcl,hcl_5702_ovc_hcl,hcl_5702_vsf_hcl,hcl_5702_vsf_hcl_error,hcl_5702_am_h2o,hcl_5702_ovc_h2o,hcl_5702_vsf_h2o,hcl_5702_vsf_h2o_error,hcl_5702_am_ch4,hcl_5702_ovc_ch4,hcl_5702_vsf_ch4,hcl_5702_vsf_ch4_error,hcl_5702_ncbf,hcl_5702_cfampocl,hcl_5702_cfperiod,hcl_5702_cfphase,hcl_5702_cbf_01,hcl_5702_cbf_02,hcl_5735_nit,hcl_5735_cl,hcl_5735_ct,hcl_5735_cc,hcl_5735_fs,hcl_5735_sg,hcl_5735_zo,hcl_5735_rmsocl,hcl_5735_zpres,hcl_5735_am_hcl,hcl_5735_ovc_hcl,hcl_5735_vsf_hcl,hcl_5735_vsf_hcl_error,hcl_5735_am_h2o,hcl_5735_ovc_h2o,hcl_5735_vsf_h2o,hcl_5735_vsf_h2o_error,hcl_5735_am_ch4,hcl_5735_ovc_ch4,hcl_5735_vsf_ch4,hcl_5735_vsf_ch4_error,hcl_5735_ncbf,hcl_5735_cfampocl,hcl_5735_cfperiod,hcl_5735_cfphase,hcl_5735_cbf_01,hcl_5735_cbf_02,hcl_5739_nit,hcl_5739_cl,hcl_5739_ct,hcl_5739_cc,hcl_5739_fs,hcl_5739_sg,hcl_5739_zo,hcl_5739_rmsocl,hcl_5739_zpres,hcl_5739_am_hcl,hcl_5739_ovc_hcl,hcl_5739_vsf_hcl,hcl_5739_vsf_hcl_error,hcl_5739_am_h2o,hcl_5739_ovc_h2o,hcl_5739_vsf_h2o,hcl_5739_vsf_h2o_error,hcl_5739_am_ch4,hcl_5739_ovc_ch4,hcl_5739_vsf_ch4,hcl_5739_vsf_ch4_error,hcl_5739_ncbf,hcl_5739_cfampocl,hcl_5739_cfperiod,hcl_5739_cfphase,hcl_5739_cbf_01,hcl_5739_cbf_02,luft_6146_nit,luft_6146_cl,luft_6146_ct,luft_6146_cc,luft_6146_fs,luft_6146_sg,luft_6146_zo,luft_6146_rmsocl,luft_6146_zpres,luft_6146_am_luft,luft_6146_ovc_luft,luft_6146_vsf_luft,luft_6146_vsf_luft_error,luft_6146_ncbf,luft_6146_cfampocl,luft_6146_cfperiod,luft_6146_cfphase";


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

            windows.insert(win_name, s);
        }
    }

    // Remove any skipped windows that also show up in windows, as those were
    // probably commented out because they conflict
    skipped_windows.retain(|el| !windows.contains_key(el));
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
fn _check_float_variable(nch: &netcdf::File, varname: &str, expected_value: f32, missing_ok: bool, clargs: &CmdLineArgs) -> Result<bool, String> {
    let nc_data = match _get_var(nch, varname) {
        Ok(data) => data,
        Err(err) => {
            if missing_ok {
                if clargs.verbosity == 3 {
                    println!("    - FAIL: variable '{}' is missing", varname);
                }
                return Ok(false);
            }else{
                return Err(err);
            }
        }
    };

    return _all_equal_float(&nc_data, expected_value, clargs);

}


fn _get_var<'a>(nch: &'a netcdf::File, varname: &str) -> Result<netcdf::Variable<'a>, String> {
    match nch.variable(varname) {
        Some(v) => return Ok(v),
        None => return Err(format!("Could not read variable '{}'", varname))
    }
}

fn _print_variable_results(varname: &str, n_total: usize, n_wrong: usize, clargs: &CmdLineArgs) -> bool {
    let is_ok = n_wrong == 0;
    if is_ok {
        if clargs.verbosity >= 3 && !clargs.failures_only{
            println!("    - PASS: {}", varname);
        }
    } else {
        if clargs.verbosity >= 3 {
            let percent = n_wrong as f32 / n_total as f32 * 100.0;
            println!("    - FAIL: {}/{} ({:.2}%) of {} have incorrect values", n_wrong, n_total, percent, varname);
        }
    }

    return is_ok;
}

fn _all_equal_float(var: &netcdf::Variable, expected_value: f32, clargs: &CmdLineArgs) -> Result<bool, String> {
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

    
    let is_ok = _print_variable_results(&var.name(), n_total, n_wrong, clargs);
    return Ok(is_ok)
}

fn _get_string_attribute_value(nch: &netcdf::File, att_name: &str, clargs: &CmdLineArgs) -> Result<String, String> {
    let att_val = match nch.attribute(att_name) {
        Some(v) => {
            match v.value() {
                Ok(inner) => inner,
                Err(err) => return Err(format!("Could not get value for attribute '{}': {}", att_name, err))
            }
        },
        None => {
            if clargs.verbosity >= 2 {
                println!("  - FAIL: attribute '{}' is not present", att_name);
            }
            return Ok(String::from(ATT_MISSING_STR))
        }
    };

    let att_val = match att_val {
        netcdf::AttrValue::Str(s) => s,
        _ => return Err(format!("Attribute '{}' has an unexpected type (expected string)", att_name))
    };

    return Ok(att_val);
}

fn _check_string_attribute_value(nch: &netcdf::File, att_name: &str, expected_value: &str, clargs: &CmdLineArgs) -> Result<bool, String> {
    let att_val = _get_string_attribute_value(nch, att_name, clargs)?;
    if att_val == ATT_MISSING_STR {
        return Ok(false)
    }

    let att_ok = att_val == expected_value;
    if att_ok {
        if !clargs.failures_only{
            if clargs.verbosity == 2 {
                println!("  - PASS: attribute '{}' has the expected value", att_name);
            }else if clargs.verbosity == 3 {
                println!("  - PASS: attribute '{}' has the expected value ('{}')", att_name, expected_value);
            }
        }
    }else{
        if clargs.verbosity >= 2 {
            println!("  - FAIL: attribute '{}' has the wrong value", att_name);
        }
        if clargs.verbosity == 3 {
            println!("      (expected = '{}', actual = '{}')", expected_value, att_val);
        }
    }

    Ok(att_ok)
}


// *************** //
// CHECK FUNCTIONS //
// *************** //

fn check_adcfs(nch: &netcdf::File, adcfs: &HashMap<&'static str, Adcf>, clargs: &CmdLineArgs) -> Result<bool, String> {
    let verbosity = clargs.verbosity;
    
    // Get the windows in alphanumeric order
    let mut windows: Vec<&'static str> = adcfs.keys().map(|x| *x).collect();
    windows.sort_unstable();

    if verbosity > 1 {
        println!("=== Checking ADCF values ===");
    }

    let mut all_ok = true;
    for window in windows {
        let win_ok = check_one_adcf(nch, window, adcfs.get(window).unwrap(), clargs)?;
        all_ok = all_ok && win_ok;
    }

    if verbosity == 1 {
        if all_ok {
            if !clargs.failures_only{ println!("* PASS: ADCFs match expected values") }; 
        }else {
            println!("* FAIL: ADCFs do not match expected values");
        }
    }
    
    Ok(all_ok)
}

fn check_one_adcf(nch: &netcdf::File, window: &str, adcf: &Adcf, clargs: &CmdLineArgs) -> Result<bool, String> {
    let verbosity = clargs.verbosity;

    if verbosity > 2 {
        println!("  * Checking {} ADCFS:", window);
    }

    let adcfs_ok = _check_float_variable(nch, &format!("{}_adcf", window), adcf.adcf, true, clargs)?;
    let errs_ok = _check_float_variable(nch, &format!("{}_adcf_error", window), adcf.err, true, clargs)?;
    let g_ok = _check_float_variable(nch, &format!("{}_g", window), adcf.g as f32, true, clargs)?;
    let p_ok = _check_float_variable(nch, &format!("{}_p", window), adcf.p as f32, true, clargs)?;

    let all_ok = adcfs_ok && errs_ok && g_ok && p_ok;

    if verbosity == 2 {
        if all_ok {
            if !clargs.failures_only{ println!("  - PASS: {} ADCFs are correct", window) };
        }else{
            println!("  - FAIL: {} ADCFS are incorrect", window);
        }
    }

    Ok(all_ok)
}

fn check_aicfs(nch: &netcdf::File, aicfs: &HashMap<&'static str, Aicf>, clargs: &CmdLineArgs) -> Result<bool, String> {
    let mut gases: Vec<&'static str> = aicfs.keys().map(|x| *x).collect();
    gases.sort_unstable();

    if clargs.verbosity > 1 {
        println!("\n=== Checking AICF values ===");
    }

    let mut all_ok = true;
    for gas in gases {
        let gas_ok = check_one_aicf(nch, gas, aicfs.get(gas).unwrap(), clargs)?;
        all_ok = all_ok && gas_ok;
    }

    if clargs.verbosity == 1 {
        if all_ok {
            if !clargs.failures_only{ println!("* PASS: AICFs match expected values") };
        }else{
            println!("* FAIL: AICFs do not match expected values");
        }
    }

    Ok(all_ok)
}


fn check_one_aicf(nch: &netcdf::File, gas: &str, aicf: &Aicf, clargs: &CmdLineArgs) -> Result<bool, String> {
    // let aicfs_ok = _all_equal_float(&nc_aicfs, aicf.aicf, verbosity)?;
    let aicfs_ok = _check_float_variable(nch, &format!("{}_aicf", gas), aicf.aicf, true, clargs)?;
    let errs_ok = _check_float_variable(nch, &format!("{}_aicf_error", gas), aicf.err, true, clargs)?;

    let all_ok = aicfs_ok && errs_ok;

    if clargs.verbosity == 2 {
        if all_ok {
            if !clargs.failures_only{ println!("  - PASS: {} AICFS are correct", gas) };
        }else{
            println!("  - FAIL: {} AICFS are not correct", gas);
        }
    }

    return Ok(all_ok);
}

fn check_window_scale_factors(nch: &netcdf::File, windows: &HashMap<String, Window>, clargs: &CmdLineArgs) -> Result<bool, String> {
    let mut win_names: Vec<&str> = windows.keys().map(|x| x.as_ref()).collect();
    win_names.sort_unstable();

    if clargs.verbosity > 1 {
        println!("\n=== Checking window-to-window scale factors ===");
    }

    let mut all_ok = true;
    for win in win_names {
        let win_ok = check_one_window_sf(nch, win, windows.get(win).unwrap(), clargs)?;
        all_ok = all_ok && win_ok;
    }

    if clargs.verbosity == 1 {
        if all_ok {
            if !clargs.failures_only{ println!("* PASS: Window-to-window scale factors match expected values") };
        }else {
            println!("* FAIL: Window-to-window scale factors do not match expected values");
        }
    }

    Ok(all_ok)
}

fn check_one_window_sf(nch: &netcdf::File, win_name: &str, window: &Window, clargs: &CmdLineArgs) -> Result<bool, String> {
    let nc_sfs = _get_var(nch, &format!("vsw_sf_{}", win_name))?;
    let sfs_ok = _all_equal_float(&nc_sfs, window.sf, clargs)?;

    if clargs.verbosity == 2 {
        if sfs_ok {
            if !clargs.failures_only {println!("  - PASS: {} window-to-window scale factors are correct", win_name)};
        }else{
            println!("  - FAIL: {} window-to-window scale factors are not correct", win_name);
        }
    }

    return Ok(sfs_ok);
}

fn check_included_windows(nch: &netcdf::File, windows: &HashMap<String, Window>, skipped_windows: &Vec<String>, clargs: &CmdLineArgs) -> Result<bool, String> {
    let mut expected_win_vars: Vec<String> = windows.keys().map(|win| format!("vsw_ada_x{}", win)).collect();
    expected_win_vars.sort_unstable();
    let mut unexpected_win_vars: Vec<String> = skipped_windows.iter().map(|win| format!("vsw_ada_x{}", win)).collect();
    unexpected_win_vars.sort_unstable();

    if clargs.verbosity > 1 {
        println!("\n=== Checking windows present ===");
    }

    let ok_expected = check_variables_present(nch, &expected_win_vars, true, clargs)?;
    let ok_unexpected = check_variables_present(nch, &unexpected_win_vars, false, clargs)?;

    if clargs.verbosity == 1 {
        if ok_expected {
            if !clargs.failures_only{println!("* PASS: All windows expected to be present are")};
        }else{
            println!("* FAIL: At least one window expected to be present is missing");
        }

        if ok_unexpected {
            if !clargs.failures_only{println!("* PASS: All windows expected to be removed are")};
        }else{
            println!("* FAIL: At least one window expected to have been removed is present");
        }
    }

    Ok(ok_expected && ok_unexpected)
}

fn check_variables_present<'a>(nch: &netcdf::File, variables: &'a[String], expected: bool, clargs: &CmdLineArgs) -> Result<bool, String> {
    // Used to check variables added or removed in Phase 2
    let mut vars_ok = true;
    for varname in variables {
        if let Some(_) = nch.variable(varname) {
            if expected {
                if clargs.verbosity >= 2 {
                    if !clargs.failures_only{ println!("  - PASS: variable '{}' is present as expected", varname) };
                }
            }else{
                vars_ok = false;
                if clargs.verbosity >= 2 {
                    println!("  - FAIL: variable '{}' is present but should not be", varname);
                }
            }
        }else{
            if expected {
                vars_ok = false;
                if clargs.verbosity >= 2 {
                    println!("  - FAIL: variable '{}' is not present but should be", varname);
                }
            }else{
                if clargs.verbosity >= 2 {
                    if !clargs.failures_only{ println!("  - PASS: variable '{}' is absent as expected", varname) };
                }
            }
        }
    }

    return Ok(vars_ok);
}

fn _check_write_netcdf_hash(nch: &netcdf::File, clargs: &CmdLineArgs) -> Result<bool, String> {
    let att_name = "code_version";
    let att_val = _get_string_attribute_value(nch, att_name, clargs)?;
    if att_val == ATT_MISSING_STR {
        if clargs.verbosity >= 2 {
            println!("  - FAIL: attribute '{}' is not present", att_name);
        }
        return Ok(false);
    }

    lazy_static! {
        static ref RE: Regex = Regex::new(r"commit ([0-9a-f]+)").unwrap();
    }

    let hash = if let Some(caps) = RE.captures(&att_val) {
        caps.get(1).unwrap().as_str()
    }else{
        return Err(format!("Could not get the write_netcdf commit hash from the attribute {}", att_name));
    };

    let hash_ok = hash == WRITE_NC_HASH;
    if hash_ok {
        if !clargs.failures_only{
            if clargs.verbosity == 2 {
                println!("  - PASS: write_netcdf hash in attribute '{}' has the expected value", att_name);
            }else if clargs.verbosity == 3 {
                println!("  - PASS: write_netcdf hash in attribute '{}' has the expected value ('{}')", att_name, WRITE_NC_HASH);
            }
        }
    }else{
        if clargs.verbosity >= 2 {
            println!("  - FAIL: write_netcdf hash in attribute '{}' has the wrong value", att_name);
        }
        if clargs.verbosity == 3 {
            println!("      (expected = '{}', actual = '{}')", WRITE_NC_HASH, hash);
        }
    }

    return Ok(hash_ok);
}

fn check_program_versions(nch: &netcdf::File, clargs: &CmdLineArgs) -> Result<bool, String> {
    if clargs.verbosity > 1 {
        println!("\n=== Checking program versions ===");
    }

    let gsetup_ok = _check_string_attribute_value(nch, "gsetup_version", GSETUP_VERSION, clargs)?;
    let gfit_ok = _check_string_attribute_value(nch, "gfit_version", GFIT_VERSION, clargs)?;
    let collate_ok = _check_string_attribute_value(nch, "collate_results_version", COLLATE_VERSION, clargs)?;
    let airmass_ok = _check_string_attribute_value(nch, "apply_airmass_correction_version", AIRMASS_VERSION, clargs)?;
    let average_ok = _check_string_attribute_value(nch, "average_results_version", AVERAGE_VERSION, clargs)?;
    let insitu_ok = _check_string_attribute_value(nch, "apply_insitu_correction_version", INSITU_VERSION, clargs)?;
    let write_nc_ok = _check_write_netcdf_hash(nch, clargs)?;

    let all_ok = gsetup_ok && gfit_ok && collate_ok && airmass_ok && average_ok && insitu_ok && write_nc_ok;

    if clargs.verbosity == 1 {
        if all_ok && !clargs.failures_only {
            println!("* PASS: All program versions match expected");
        }else if !all_ok {
            println!("* FAIL: At least one program version does not match expected");
        }
    }

    Ok(all_ok)
}

fn check_ingaas_variables(nch: &netcdf::File, clargs: &CmdLineArgs) -> Result<bool, String> {
    let variable_list: Vec<&str> = EXPECTED_INGAAS_VARS.split(',').collect();
    let ntotal = variable_list.len();
    let mut nmissing = 0;

    if clargs.verbosity > 1 {
        println!("\n=== Checking InGaAs variables ===");
    }

    for varname in variable_list {
        if let None = nch.variable(varname) {
            nmissing += 1;
            if clargs.verbosity >= 3 {
                if clargs.verbosity == 4 || nmissing < 11 {
                    println!("    - FAIL: variable is {} missing", varname);
                }else if nmissing == 11 {
                    println!("    (further missing variables omitted)");
                }
            }
        }
    }

    if clargs.verbosity >= 1 {
        if nmissing == 0 && !clargs.failures_only {
            println!("* PASS: All expected InGaAs variables present");
        }else if nmissing > 0 {
            println!("* FAIL: {}/{} expected InGaAs variables missing", nmissing, ntotal);
        }
    }

    Ok(nmissing == 0)
}


fn driver(nc_file: &str, clargs: &CmdLineArgs) -> Result<bool, String> {
    
    let adcfs = read_adcf_table();
    let aicfs = read_aicf_table();
    let (windows, skipped_windows) = read_windows_table();

    let nch = match netcdf::open(nc_file) {
        Ok(h) => h,
        Err(err) => return Err(format!("Unable to open {}: {}", nc_file, err))
    };

    let adcfs_ok = check_adcfs(&nch, &adcfs, clargs)?;
    let aicfs_ok = check_aicfs(&nch, &aicfs, clargs)?;
    let sfs_ok = check_window_scale_factors(&nch, &windows, clargs)?;
    let windows_ok = check_included_windows(&nch, &windows, &skipped_windows, clargs)?;
    let versions_ok = check_program_versions(&nch, clargs)?;
    let ingaas_ok = check_ingaas_variables(&nch, clargs)?;

    let overall_ok = adcfs_ok && aicfs_ok && sfs_ok && windows_ok && versions_ok && ingaas_ok;
    if clargs.verbosity >= 0 {
        if clargs.verbosity > 0 {println!("");}

        if overall_ok {
            println!("{} PASSES all tests - it appears to be a correct Phase 2 file", nc_file);
        }else{
            println!("{} FAILS at least one test - it may be a Phase 1 file or there was a problem in processing.", nc_file);
        }
    }
    
    return Ok(overall_ok);
}

#[derive(Debug)]
struct CmdLineArgs {
    nc_file: String,
    verbosity: i8,
    failures_only: bool
}

fn parse_clargs() -> CmdLineArgs {
    let yml = clap::load_yaml!("clargs.yml");
    let clargs = clap::App::from_yaml(yml).version(clap::crate_version!()).get_matches();

    let nc_file = clargs.value_of("nc_file").unwrap();
    let nverb = clargs.occurrences_of("verbose");
    let nquiet = clargs.occurrences_of("quiet");
    let failures_only = clargs.occurrences_of("failures_only") > 0;

    let args = CmdLineArgs{
        nc_file: String::from(nc_file),
        verbosity: if nquiet > 0 {-1} else {nverb as i8},
        failures_only: failures_only
    };

    return args;
}

fn main() {
    let clargs = parse_clargs();

    match driver(&clargs.nc_file, &clargs) {
        Ok(passes) => {
            if passes {std::process::exit(0);}
            else {std::process::exit(1);}
        },
        Err(msg) => {
            eprintln!("ERROR: {}", msg);
            std::process::exit(2);
        }
    }
}
