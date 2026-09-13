# OrcaSlicer printer smoke summary

745 of 1001 printers pass the strict ordered-byte comparison (generator identity/timestamp lines normalized; classic wall generator baseline; cube model).

Statuses: `PASS` (normalized byte equality), `DIVERGENT` (first byte difference), `ORCA_ERROR` (the upstream OrcaSlicer 2.4.2 reference binary itself failed, so no reference stream exists), `VENDOR_INCOMPLETE` (the vendor profile tree does not ship the machine's referenced default process preset), `ARES_ERROR` (Ares failed to load or slice the case).

| status | printer | first divergence |
|---|---|---|
| PASS | Afinia/Afinia H+1(HS) 0.4 nozzle |  |
| PASS | Afinia/Afinia H+1(HS) 0.6 nozzle |  |
| PASS | Anker/Anker M5 0.2 nozzle |  |
| PASS | Anker/Anker M5 0.25 nozzle |  |
| PASS | Anker/Anker M5 0.4 nozzle |  |
| PASS | Anker/Anker M5 0.6 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.2 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.25 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.4 nozzle |  |
| PASS | Anker/Anker M5 All-Metal 0.6 nozzle |  |
| PASS | Anker/Anker M5C 0.2 nozzle |  |
| PASS | Anker/Anker M5C 0.25 nozzle |  |
| PASS | Anker/Anker M5C 0.4 nozzle |  |
| PASS | Anker/Anker M5C 0.6 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 0.4 nozzle |  |
| PASS | Anycubic/Anycubic 4Max Pro 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Chiron 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 2 Neo 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 2 Plus 0.4 nozzle | first difference at byte 21329 (line 861, column 29; expected 107897 bytes, actual 107895 bytes) context:   line 860: "G1 X155.95 Y156.231 E1.8846"   line 861: expected "G1 X156.118 Y156.891 E1.90651"; actual "G1 X156.118 Y156.891 E1.9065"   line 862: "G1 X156.891 Y157.665 E1.94168" |
| PASS | Anycubic/Anycubic Kobra 2 Pro 0.4 nozzle |  |
| DIVERGENT | Anycubic/Anycubic Kobra 3 0.2 nozzle | first difference at byte 18994 (line 661, column 29; expected 454235 bytes, actual 454235 bytes) context:   line 660: ";WIPE_START"   line 661: expected "G1 X121.761 Y120.566 E-.70072"; actual "G1 X121.761 Y120.566 E-.70071"   line 662: expected "G1 X121.88 Y120.53 E-.09928"; actual "G1 X121.88 Y120.53 E-.09929" |
| PASS | Anycubic/Anycubic Kobra 3 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra 3 Max 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Neo 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra Plus 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.25 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.6 nozzle |  |
| PASS | Anycubic/Anycubic Kobra S1 Max 0.8 nozzle |  |
| PASS | Anycubic/Anycubic Kobra X 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Predator 0.4 nozzle |  |
| PASS | Anycubic/Anycubic Vyper 0.4 nozzle |  |
| PASS | Anycubic/Anycubic i3 Mega S 0.4 nozzle |  |
| PASS | Artillery/Artillery Genius 0.4 nozzle |  |
| PASS | Artillery/Artillery Genius Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery Hornet 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.2 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.6 nozzle |  |
| PASS | Artillery/Artillery M1 Pro 0.8 nozzle |  |
| DIVERGENT | Artillery/Artillery Sidewinder X1 0.4 nozzle | first difference at byte 4008 (line 167, column 1; expected 104247 bytes, actual 104247 bytes) context:   line 166: "G1 X154.75 Y145.25 E.43192"   line 167: expected "M73 P4 R8"; actual "G1 X154.75 Y154.71 E.4301"   line 168: expected "G1 X154.75 Y154.71 E.4301"; actual "M73 P4 R8" |
| DIVERGENT | Artillery/Artillery Sidewinder X2 0.4 nozzle | first difference at byte 2306 (line 97, column 1; expected 104344 bytes, actual 104344 bytes) context:   line 96: "G1 X156.641 Y157.649 E.0712"   line 97: expected "M73 P36 R8"; actual "G1 X155 Y158.116 E.07757"   line 98: expected "G1 X155 Y158.116 E.07757"; actual "M73 P36 R8" |
| PASS | Artillery/Artillery Sidewinder X3 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X3 Pro 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Plus 0.4 nozzle |  |
| PASS | Artillery/Artillery Sidewinder X4 Pro 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 0.8 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.2 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.4 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.6 nozzle |  |
| PASS | BBL/Bambu Lab A1 mini 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab H2D 0.2 nozzle | first difference at byte 8629 (line 105, column 24; expected 403760 bytes, actual 403760 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D 0.4 nozzle | first difference at byte 8605 (line 105, column 24; expected 162460 bytes, actual 162460 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D 0.6 nozzle | first difference at byte 8625 (line 105, column 24; expected 120090 bytes, actual 120090 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D 0.8 nozzle | first difference at byte 8627 (line 105, column 24; expected 103783 bytes, actual 103783 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.2 nozzle | first difference at byte 8642 (line 105, column 24; expected 404635 bytes, actual 404635 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.4 nozzle | first difference at byte 8618 (line 105, column 24; expected 162935 bytes, actual 162935 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.6 nozzle | first difference at byte 8626 (line 105, column 24; expected 119636 bytes, actual 119636 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| DIVERGENT | BBL/Bambu Lab H2D Pro 0.8 nozzle | first difference at byte 8640 (line 105, column 24; expected 104058 bytes, actual 104058 bytes) context:   line 104: "; enable_pressure_advance = 0"   line 105: expected "; enable_prime_tower = 1"; actual "; enable_prime_tower = 0"   line 106: "; enable_support = 0" |
| PASS | BBL/Bambu Lab H2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab H2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab P1P 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 371083 bytes, actual 371106 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.4 nozzle | first difference at byte 93 (line 3, column 28; expected 143837 bytes, actual 143848 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 37s; total estimated time: 11m 52s"; actual "; model printing time: 5m 39s; total estimated time: 11m 54s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 101993 bytes, actual 101993 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 42s; total estimated time: 9m 56s"; actual "; model printing time: 3m 44s; total estimated time: 9m 57s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1P 0.8 nozzle | first difference at byte 93 (line 3, column 28; expected 87104 bytes, actual 87104 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 43s; total estimated time: 9m 0s"; actual "; model printing time: 2m 44s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 371249 bytes, actual 371272 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 15m 25s"; actual "; model printing time: 8m 39s; total estimated time: 15m 28s"   line 4: "; estimated first layer printing time (normal mode) = 6m 48s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 143988 bytes, actual 143988 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 5s; total estimated time: 10m 20s"; actual "; model printing time: 4m 8s; total estimated time: 10m 23s"   line 4: "; estimated first layer printing time (normal mode) = 6m 14s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 102134 bytes, actual 102145 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 9m 0s"; actual "; model printing time: 2m 48s; total estimated time: 9m 2s"   line 4: "; estimated first layer printing time (normal mode) = 6m 13s" |
| DIVERGENT | BBL/Bambu Lab P1S 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 87316 bytes, actual 87327 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 7s; total estimated time: 8m 25s"; actual "; model printing time: 2m 9s; total estimated time: 8m 27s"   line 4: "; estimated first layer printing time (normal mode) = 6m 17s" |
| PASS | BBL/Bambu Lab P2S 0.2 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.4 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.6 nozzle |  |
| PASS | BBL/Bambu Lab P2S 0.8 nozzle |  |
| DIVERGENT | BBL/Bambu Lab X1 0.2 nozzle | first difference at byte 92 (line 3, column 27; expected 396001 bytes, actual 395885 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 169245 bytes, actual 169164 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 5m 38s; total estimated time: 12m 54s"; actual "; model printing time: 5m 40s; total estimated time: 12m 57s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 127211 bytes, actual 127139 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 3m 43s; total estimated time: 11m 1s"; actual "; model printing time: 3m 45s; total estimated time: 11m 2s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 0.8 nozzle | first difference at byte 93 (line 3, column 28; expected 112013 bytes, actual 111945 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 44s; total estimated time: 10m 7s"; actual "; model printing time: 2m 45s; total estimated time: 10m 9s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.2 nozzle | first difference at byte 92 (line 3, column 27; expected 396062 bytes, actual 395946 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 37s; total estimated time: 18m 36s"; actual "; model printing time: 8m 40s; total estimated time: 18m 39s"   line 4: "; estimated first layer printing time (normal mode) = 9m 58s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 169250 bytes, actual 169180 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 7s; total estimated time: 11m 23s"; actual "; model printing time: 4m 9s; total estimated time: 11m 25s"   line 4: "; estimated first layer printing time (normal mode) = 7m 16s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 127323 bytes, actual 127220 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 47s; total estimated time: 10m 4s"; actual "; model printing time: 2m 49s; total estimated time: 10m 6s"   line 4: "; estimated first layer printing time (normal mode) = 7m 17s" |
| DIVERGENT | BBL/Bambu Lab X1 Carbon 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 112166 bytes, actual 112077 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 9s; total estimated time: 9m 32s"; actual "; model printing time: 2m 10s; total estimated time: 9m 34s"   line 4: "; estimated first layer printing time (normal mode) = 7m 23s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.2 nozzle | first difference at byte 93 (line 3, column 28; expected 394389 bytes, actual 394309 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 8m 36s; total estimated time: 19m 11s"; actual "; model printing time: 8m 39s; total estimated time: 19m 15s"   line 4: "; estimated first layer printing time (normal mode) = 10m 35s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.4 nozzle | first difference at byte 92 (line 3, column 27; expected 167733 bytes, actual 167663 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 4m 6s; total estimated time: 11m 33s"; actual "; model printing time: 4m 8s; total estimated time: 11m 35s"   line 4: "; estimated first layer printing time (normal mode) = 7m 27s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.6 nozzle | first difference at byte 93 (line 3, column 28; expected 125678 bytes, actual 125595 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 46s; total estimated time: 10m 17s"; actual "; model printing time: 2m 48s; total estimated time: 10m 19s"   line 4: expected "; estimated first layer printing time (normal mode) = 7m 30s"; actual "; estimated first layer printing time (normal mode) = 7m 31s" |
| DIVERGENT | BBL/Bambu Lab X1E 0.8 nozzle | first difference at byte 92 (line 3, column 27; expected 110611 bytes, actual 110521 bytes) context:   line 2: "; generated by <SLICER> 2.4.2 on <TIMESTAMP>"   line 3: expected "; model printing time: 2m 8s; total estimated time: 9m 45s"; actual "; model printing time: 2m 9s; total estimated time: 9m 47s"   line 4: "; estimated first layer printing time (normal mode) = 7m 37s" |
| DIVERGENT | BBL/Bambu Lab X2D 0.2 nozzle | first difference at byte 11463 (line 146, column 13; expected 409762 bytes, actual 409871 bytes) context:   line 145: "; filament_density = 1.26"   line 146: expected "; filament_diameter = 1.75"; actual "; filament_deretraction_speed = 50;50"   line 147: expected "; filament_end_gcode = \"; filament end gcode \\n\\n\""; actual "; filament_diameter = 1.75" |
| DIVERGENT | BBL/Bambu Lab X2D 0.4 nozzle | first difference at byte 59612 (line 1195, column 15; expected 163233 bytes, actual 163233 bytes) context:   line 1194: ""   line 1195: expected "M9711 M0 E1 X186 Y130 Z0.6 S11 C10 O0 T3000"; actual "M9711 M0 E1 X140 Y175 Z0.6 S11 C10 O0 T3000"   line 1196: "" |
| DIVERGENT | BBL/Bambu Lab X2D 0.6 nozzle | first difference at byte 59255 (line 1181, column 15; expected 121840 bytes, actual 121840 bytes) context:   line 1180: ""   line 1181: expected "M9711 M0 E1 X186 Y130 Z0.7 S11 C10 O0 T3000"; actual "M9711 M0 E1 X140 Y175 Z0.7 S11 C10 O0 T3000"   line 1182: "" |
| DIVERGENT | BBL/Bambu Lab X2D 0.8 nozzle | first difference at byte 59037 (line 1179, column 15; expected 106134 bytes, actual 106134 bytes) context:   line 1178: ""   line 1179: expected "M9711 M0 E1 X186 Y130 Z0.8 S11 C10 O0 T3000"; actual "M9711 M0 E1 X140 Y175 Z0.8 S11 C10 O0 T3000"   line 1180: "" |
| PASS | BIQU/BIQU B1 (0.4 nozzle) |  |
| PASS | BIQU/BIQU BX (0.4 nozzle) |  |
| PASS | BIQU/BIQU Hurakan (0.4 nozzle) |  |
| PASS | Blocks/BLOCKS Pro S100 0.4 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 0.6 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 0.8 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.0 nozzle |  |
| PASS | Blocks/BLOCKS Pro S100 1.2 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.4 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RD50 V2 0.8 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.4 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.6 nozzle |  |
| PASS | Blocks/BLOCKS RF50 0.8 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 0.4 nozzle |  |
| PASS | CONSTRUCT3D/Construct 1 XL 0.6 nozzle |  |
| PASS | Chuanying/Chuanying X1 0.25 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.4 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.6 Nozzle |  |
| PASS | Chuanying/Chuanying X1 0.8 Nozzle |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle - Ender-3 V3 Plus |  |
| PASS | Co Print/Co Print ChromaSet 0.4 nozzle fast |  |
| PASS | CoLiDo/CoLiDo 160 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo DIY 4.0 V2 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo SR1 0.4 nozzle |  |
| PASS | CoLiDo/CoLiDo X16 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T300 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.4 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.6 nozzle |  |
| PASS | Comgrow/Comgrow T500 0.8 nozzle |  |
| PASS | Creality/Creality CR-10 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.2 nozzle |  |
| DIVERGENT | Creality/Creality CR-10 SE 0.4 nozzle | first difference at byte 55029 (line 2554, column 1; expected 89542 bytes, actual 89542 bytes) context:   line 2553: "G1 X114.79 Y114.75 E.29314"   line 2554: expected "G1 E-.61523 F1800"; actual "M73 P88 R0"   line 2555: expected "M73 P88 R0"; actual "G1 E-.61523 F1800" |
| PASS | Creality/Creality CR-10 SE 0.6 nozzle |  |
| PASS | Creality/Creality CR-10 SE 0.8 nozzle |  |
| PASS | Creality/Creality CR-10 V2 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 V3 0.4 nozzle |  |
| PASS | Creality/Creality CR-10 V3 0.6 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 Max 0.2 nozzle | first difference at byte 7041 (line 291, column 29; expected 394066 bytes, actual 394066 bytes) context:   line 290: "G1 X195.605 Y204.196 E-.58892"   line 291: expected "G1 X195.605 Y203.947 E-.37351"; actual "G1 X195.605 Y203.947 E-.37352"   line 292: "G1 X195.858 Y204.2 E-.53756" |
| PASS | Creality/Creality CR-6 Max 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 Max 0.8 nozzle |  |
| DIVERGENT | Creality/Creality CR-6 SE 0.2 nozzle | first difference at byte 7068 (line 292, column 29; expected 394163 bytes, actual 394163 bytes) context:   line 291: "G1 X113.105 Y121.696 E-.58892"   line 292: expected "G1 X113.105 Y121.447 E-.37351"; actual "G1 X113.105 Y121.447 E-.37352"   line 293: "G1 X113.358 Y121.7 E-.53756" |
| PASS | Creality/Creality CR-6 SE 0.4 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.6 nozzle |  |
| PASS | Creality/Creality CR-6 SE 0.8 nozzle |  |
| PASS | Creality/Creality CR-M4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 Pro 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Plus 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 S1 Pro 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V2 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V2 Neo 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 0.6 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 KE 0.2 nozzle | first difference at byte 279 (line 11, column 35; expected 171903 bytes, actual 171903 bytes) context:   line 10: "; external perimeters extrusion width = 0.21mm"   line 11: expected "; perimeters extrusion width = 0.23mm"; actual "; perimeters extrusion width = 0.22mm"   line 12: expected "; infill extrusion width = 0.23mm"; actual "; infill extrusion width = 0.22mm" |
| PASS | Creality/Creality Ender-3 V3 KE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 KE 0.8 nozzle |  |
| DIVERGENT | Creality/Creality Ender-3 V3 Plus 0.4 nozzle | first difference at byte 22406 (line 893, column 1; expected 123624 bytes, actual 123624 bytes) context:   line 892: "G1 X146.052 Y146.052 E.26192"   line 893: expected "M73 P21 R4"; actual "G1 X153.948 Y146.052 E.26192"   line 894: expected "G1 X153.948 Y146.052 E.26192"; actual "M73 P21 R4" |
| PASS | Creality/Creality Ender-3 V3 Plus 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.2 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.4 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.6 nozzle |  |
| PASS | Creality/Creality Ender-3 V3 SE 0.8 nozzle |  |
| PASS | Creality/Creality Ender-3 V4 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Max 0.8 nozzle |  |
| PASS | Creality/Creality Ender-5 Plus 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.2 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.25 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.3 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.5 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.6 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 0.8 nozzle |  |
| PASS | Creality/Creality Ender-5 Pro (2019) 1.0 nozzle |  |
| PASS | Creality/Creality Ender-5 S1 0.4 nozzle |  |
| PASS | Creality/Creality Ender-5S 0.4 nozzle |  |
| PASS | Creality/Creality Ender-6 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.2 nozzle |  |
| PASS | Creality/Creality Hi 0.4 nozzle |  |
| PASS | Creality/Creality Hi 0.6 nozzle |  |
| PASS | Creality/Creality Hi 0.8 nozzle |  |
| PASS | Creality/Creality K1 (0.4 nozzle) |  |
| PASS | Creality/Creality K1 (0.6 nozzle) |  |
| PASS | Creality/Creality K1 (0.8 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.4 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.6 nozzle) |  |
| PASS | Creality/Creality K1 Max (0.8 nozzle) |  |
| DIVERGENT | Creality/Creality K1 Max_CFS-C 0.4 nozzle | first difference at byte 91995 (line 3951, column 8; expected 112889 bytes, actual 112865 bytes) context:   line 3950: "END_PRINT"   line 3951: expected "M141 S0;set chamber_temperature"; actual "M141 S0"   line 3952: "M73 P100 R0" |
| PASS | Creality/Creality K1 SE 0.4 nozzle |  |
| PASS | Creality/Creality K1 SE 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Creality/Creality K1 SE 0.8 nozzle | Creality/Creality K1 SE 0.8 nozzle process: default preset "0.40mm Standard @Creality K1 SE 0.8 nozzle" not found |
| PASS | Creality/Creality K1 SE_CFS-C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.4 nozzle |  |
| PASS | Creality/Creality K1C 0.6 nozzle |  |
| PASS | Creality/Creality K1C 0.8 nozzle |  |
| DIVERGENT | Creality/Creality K1C_CFS-C 0.4 nozzle | first difference at byte 96253 (line 3952, column 8; expected 117121 bytes, actual 117097 bytes) context:   line 3951: "END_PRINT"   line 3952: expected "M141 S0;set chamber_temperature"; actual "M141 S0"   line 3953: "M73 P100 R0" |
| DIVERGENT | Creality/Creality K1_CFS-C 0.4 nozzle | first difference at byte 91994 (line 3950, column 8; expected 112859 bytes, actual 112835 bytes) context:   line 3949: "END_PRINT"   line 3950: expected "M141 S0;set chamber_temperature"; actual "M141 S0"   line 3951: "M73 P100 R0" |
| PASS | Creality/Creality K2 0.2 nozzle |  |
| PASS | Creality/Creality K2 0.4 nozzle |  |
| PASS | Creality/Creality K2 0.6 nozzle |  |
| PASS | Creality/Creality K2 0.8 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.2 nozzle |  |
| PASS | Creality/Creality K2 Plus 0.4 nozzle |  |
| DIVERGENT | Creality/Creality K2 Plus 0.6 nozzle | first difference at byte 3372 (line 142, column 1; expected 95769 bytes, actual 95769 bytes) context:   line 141: "G1 X174.52 Y178.568 E.29616"   line 142: expected "G1 X173.71 Y178.568 E.05491"; actual "M73 P16 R3"   line 143: expected "M73 P16 R3"; actual "G1 X173.71 Y178.568 E.05491" |
| DIVERGENT | Creality/Creality K2 Plus 0.8 nozzle | first difference at byte 5106 (line 218, column 1; expected 79002 bytes, actual 79002 bytes) context:   line 217: "G1 X175.467 Y178.313 E.38216"   line 218: expected "M73 P20 R2"; actual "G1 X174.404 Y178.313 E.1009"   line 219: expected "G1 X174.404 Y178.313 E.1009"; actual "M73 P20 R2" |
| PASS | Creality/Creality K2 Pro 0.2 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.4 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.6 nozzle |  |
| PASS | Creality/Creality K2 Pro 0.8 nozzle |  |
| PASS | Creality/Creality K2 SE 0.4 nozzle |  |
| DIVERGENT | Creality/Creality SPARKX i7 0.2 nozzle | first difference at byte 22156 (line 808, column 1; expected 361674 bytes, actual 361673 bytes) context:   line 807: "G1 X134.126 Y129.358 E.00227"   line 808: expected "M73 P8 R13"; actual "G1 X129.358 Y134.126 E.05455"   line 809: expected "G1 X129.358 Y134.126 E.05455"; actual "G1 X129.077 Y134.126 E.00227" |
| DIVERGENT | Creality/Creality SPARKX i7 0.4 nozzle | first difference at byte 2827 (line 132, column 1; expected 137363 bytes, actual 137362 bytes) context:   line 131: "G1 X129.221 Y126.2 E.26305"   line 132: expected "M73 P9 R6"; actual "G1 X128.55 Y126.2 E.02728"   line 133: expected "G1 X128.55 Y126.2 E.02728"; actual "M73 P9 R6" |
| DIVERGENT | Creality/Creality SPARKX i7 0.6 nozzle | first difference at byte 2217 (line 107, column 1; expected 104151 bytes, actual 104148 bytes) context:   line 106: "G1 X134.123 Y127.322 E.1484"   line 107: expected "M73 P13 R4"; actual "G1 X134.123 Y128.122 E.05811"   line 108: expected "G1 X134.123 Y128.122 E.05811"; actual "M73 P13 R4" |
| DIVERGENT | Creality/Creality SPARKX i7 0.8 nozzle | first difference at byte 954 (line 46, column 7; expected 88493 bytes, actual 88503 bytes) context:   line 45: "G1 X115 E.3742  F1600"   line 46: expected "M73 P15 R3"; actual "M73 P14 R3"   line 47: "G1 X110 E.3742  F6400" |
| DIVERGENT | Creality/Creality Sermoon V1 0.4 nozzle | first difference at byte 703 (line 27, column 8; expected 162827 bytes, actual 124178 bytes) context:   line 26: "M205 X8.00 Y8.00 Z0.40 E5.00 ; sets the jerk limits, mm/sec"   line 27: expected "M106 S0 ; disable fan"; actual "M106 S0"   line 28: expected "M106 P2 S0 ; disable additional fan "; actual "M106 P2 S0" |
| ORCA_ERROR | Cubicon/Cubicon xCeler-I 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-6hMDtS") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Mini 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-FIhFNv") |
| ORCA_ERROR | Cubicon/Cubicon xCeler-Plus 0.4 nozzle | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-X0i9ah") |
| PASS | Custom/MyKlipper 0.2 nozzle |  |
| PASS | Custom/MyKlipper 0.4 nozzle |  |
| PASS | Custom/MyKlipper 0.6 nozzle |  |
| PASS | Custom/MyKlipper 0.8 nozzle |  |
| PASS | Custom/MyMarlin 0.4 nozzle |  |
| PASS | Custom/MyRRF 0.4 nozzle |  |
| PASS | Custom/MyRepetier 0.4 nozzle |  |
| DIVERGENT | Custom/MyToolChanger 0.2 nozzle | first difference at byte 1295 (line 63, column 1; expected 462943 bytes, actual 462963 bytes) context:   line 62: ""   line 63: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F1800"   line 64: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Custom/MyToolChanger 0.4 nozzle | first difference at byte 1314 (line 65, column 1; expected 133736 bytes, actual 133767 bytes) context:   line 64: ""   line 65: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F1800"   line 66: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Custom/MyToolChanger 0.6 nozzle | first difference at byte 1314 (line 65, column 1; expected 118518 bytes, actual 118538 bytes) context:   line 64: ""   line 65: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F1800"   line 66: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Custom/MyToolChanger 0.8 nozzle | first difference at byte 1314 (line 65, column 1; expected 127095 bytes, actual 127115 bytes) context:   line 64: ""   line 65: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F1800"   line 66: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| PASS | DeltaMaker/DeltaMaker 2 0.35 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2T 0.5 nozzle |  |
| PASS | DeltaMaker/DeltaMaker 2XT 0.5 nozzle |  |
| PASS | Dremel/Dremel 3D20 0.4 nozzle |  |
| DIVERGENT | Dremel/Dremel 3D40 0.4 nozzle | first difference at byte 4719 (line 207, column 28; expected 104124 bytes, actual 104105 bytes) context:   line 206: "M73 P4 R9"   line 207: expected "G1 X-14.935 Y-3.413 E3.33905"; actual "G1 X-14.935 Y-3.413 E3.33904"   line 208: "G1 X-15.63 Y-3.413 E3.35298" |
| DIVERGENT | Dremel/Dremel 3D45 0.4 nozzle | first difference at byte 4987 (line 228, column 26; expected 113300 bytes, actual 113296 bytes) context:   line 227: "G1 X-15.488 Y-3.738 E3.38097"   line 228: expected "G1 X-11.262 Y.488 E3.47422"; actual "G1 X-11.262 Y.488 E3.47421"   line 229: "G1 X-11.262 Y1.029 E3.48267" |
| PASS | Elegoo/Elegoo Centauri 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Centauri Carbon 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2D 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 2S 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 3 Pro 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Max 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Plus 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.2 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 0.8 nozzle |  |
| PASS | Elegoo/Elegoo Neptune 4 Pro 1.0 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.4 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.6 nozzle |  |
| PASS | Elegoo/Elegoo Neptune X 0.8 nozzle |  |
| DIVERGENT | Elegoo/Elegoo OrangeStorm Giga 0.4 nozzle | first difference at byte 30273 (line 1528, column 28; expected 109985 bytes, actual 109985 bytes) context:   line 1527: "G1 X400.95 Y406.257 E1.38027"   line 1528: expected "G1 X401.186 Y406.55 E1.39273"; actual "G1 X401.186 Y406.55 E1.39272"   line 1529: "G1 X401.243 Y406.55 E1.39462" |
| DIVERGENT | Elegoo/Elegoo OrangeStorm Giga 0.6 nozzle | first difference at byte 5260 (line 319, column 29; expected 76231 bytes, actual 76231 bytes) context:   line 318: "G1 X405.104 Y398.838 E7.46076"   line 319: expected "G1 X404.318 Y398.838 E7.51413"; actual "G1 X404.318 Y398.838 E7.51412"   line 320: "M73 P18 R3" |
| DIVERGENT | Elegoo/Elegoo OrangeStorm Giga 0.8 nozzle | first difference at byte 10972 (line 612, column 29; expected 64957 bytes, actual 64957 bytes) context:   line 611: "G1 X402.67 Y405.732 E1.55137"   line 612: expected "G1 X404.402 Y399.268 E2.35202"; actual "G1 X404.402 Y399.268 E2.35201"   line 613: "G1 X403.768 Y399.268 E2.42783" |
| DIVERGENT | Elegoo/Elegoo OrangeStorm Giga 1.0 nozzle | first difference at byte 4211 (line 285, column 29; expected 55298 bytes, actual 55298 bytes) context:   line 284: "G1 X404.107 Y405.349 E3.0606"   line 285: expected "G1 X407.849 Y401.607 E4.06992"; actual "G1 X407.849 Y401.607 E4.06991"   line 286: "G1 X407.849 Y400.283 E4.32247" |
| PASS | Eryone/Eryone ER20 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 0.5 nozzle | first difference at byte 2090 (line 90, column 7; expected 112154 bytes, actual 113351 bytes) context:   line 89: "G1 X125.739 Y114.239"   line 90: expected "G1 X126.225 Y114.725"; actual "G1 X125.739 Y114.239"   line 91: expected "M205 X9 Y9"; actual "G1 X126.225 Y114.725" |
| DIVERGENT | Eryone/Eryone ER20 0.6 nozzle | first difference at byte 2079 (line 90, column 7; expected 93467 bytes, actual 94160 bytes) context:   line 89: "G1 X125.667 Y114.167"   line 90: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 91: expected "M205 X9 Y9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 0.8 nozzle | first difference at byte 2110 (line 92, column 7; expected 67425 bytes, actual 67929 bytes) context:   line 91: "G1 X125.399 Y113.899"   line 92: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 93: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| PASS | Eryone/Eryone ER20 Klipper 0.2 nozzle |  |
| PASS | Eryone/Eryone ER20 Klipper 0.4 nozzle |  |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.5 nozzle | first difference at byte 1564 (line 60, column 7; expected 144405 bytes, actual 145287 bytes) context:   line 59: "G1 X125.784 Y114.284"   line 60: expected "G1 X126.24 Y114.74"; actual "G1 X125.784 Y114.284"   line 61: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.24 Y114.74" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.6 nozzle | first difference at byte 1571 (line 61, column 7; expected 114496 bytes, actual 115504 bytes) context:   line 60: "G1 X125.667 Y114.167"   line 61: expected "G1 X126.19 Y114.69"; actual "G1 X125.667 Y114.167"   line 62: expected "SET_VELOCITY_LIMIT ACCEL=500 ACCEL_TO_DECEL=250 SQUARE_CORNER_VELOCITY=9"; actual "G1 X126.19 Y114.69" |
| DIVERGENT | Eryone/Eryone ER20 Klipper 0.8 nozzle | first difference at byte 1602 (line 63, column 7; expected 80922 bytes, actual 81741 bytes) context:   line 62: "G1 X125.399 Y113.899"   line 63: expected "G1 X126.09 Y114.59"; actual "G1 X125.399 Y113.899"   line 64: expected "G1 Z.4"; actual "G1 X126.09 Y114.59" |
| PASS | Eryone/Thinker X400 0.2 nozzle |  |
| PASS | Eryone/Thinker X400 0.4 nozzle |  |
| PASS | FLSun/FLSun Q5 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun QQ-S Pro 0.4 nozzle | first difference at byte 37122 (line 1903, column 1; expected 103931 bytes, actual 103931 bytes) context:   line 1902: "G1 X4.325 Y4.325 F9000"   line 1903: expected "M73 P48 R3"; actual "G1 F1148"   line 1904: expected "G1 F1148"; actual "M73 P48 R3" |
| PASS | FLSun/FLSun S1 0.4 nozzle |  |
| DIVERGENT | FLSun/FLSun Super Racer 0.4 nozzle | first difference at byte 2574 (line 105, column 1; expected 104847 bytes, actual 104847 bytes) context:   line 104: "G1 X5 Y10.611 E.08056"   line 105: expected "M73 P5 R7"; actual "G1 X-5 Y10.611 E.33172"   line 106: expected "G1 X-5 Y10.611 E.33172"; actual "G1 X-7.072 Y10.214 E.06996" |
| PASS | FLSun/FLSun T1 0.4 nozzle |  |
| PASS | FLSun/FLSun V400 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.25 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.4 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.6 nozzle |  |
| PASS | Flashforge/Flashforge AD5X 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.4 Nozzle | first difference at byte 5552 (line 276, column 15; expected 106902 bytes, actual 107888 bytes) context:   line 275: "G1 X4.464 Y4.464 F4800"   line 276: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 277: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| DIVERGENT | Flashforge/Flashforge Adventurer 3 Series 0.6 Nozzle | first difference at byte 4936 (line 253, column 9; expected 71444 bytes, actual 72396 bytes) context:   line 252: "G1 X4.197 Y4.197 F6000"   line 253: expected "G1 X4.196 Y4.164"; actual "G1 X4.197 Y4.197"   line 254: expected "G1 X4.164 Y4.164"; actual "G1 X4.196 Y4.164" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.3 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series 0.4 Nozzle | first difference at byte 5126 (line 242, column 15; expected 92980 bytes, actual 94238 bytes) context:   line 241: "G1 X4.464 Y4.464 F4800"   line 242: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 243: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 4 Series 0.6 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Adventurer 4 Series HS Nozzle | first difference at byte 5404 (line 256, column 15; expected 103767 bytes, actual 105025 bytes) context:   line 255: "G1 X4.464 Y4.464 F9000"   line 256: expected "G1 X4.464 Y4.443"; actual "G1 X4.464 Y4.464"   line 257: expected "G1 X4.443 Y4.443"; actual "G1 X4.464 Y4.443" |
| PASS | Flashforge/Flashforge Adventurer 5M 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M 0.8 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.25 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Adventurer 5M Pro 0.8 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Artemis 0.4 Nozzle | first difference at byte 5282 (line 252, column 15; expected 110781 bytes, actual 112430 bytes) context:   line 251: "G1 X4.458 Y4.458 F6000"   line 252: expected "G1 X4.458 Y4.439"; actual "G1 X4.458 Y4.458"   line 253: expected "G1 X4.439 Y4.439"; actual "G1 X4.458 Y4.439" |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-0MV7qD") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-aqOuD6") |
| ORCA_ERROR | Flashforge/Flashforge Creator 5 0.8 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-V6SdzA") |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Creator 5 Pro 0.8 nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider 2s 0.4 nozzle | first difference at byte 1642 (line 72, column 22; expected 83421 bytes, actual 83415 bytes) context:   line 71: "M204 S200"   line 72: expected "G1 X13.152 Y-6.782 F4800"; actual "G1 X13.152 Y-6.782 F4200"   line 73: expected "G1 Z.27 F4200"; actual "G1 Z.27" |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.4 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.6 Nozzle |  |
| PASS | Flashforge/Flashforge Guider 3 Ultra 0.8 Nozzle |  |
| DIVERGENT | Flashforge/Flashforge Guider4 0.25 nozzle | first difference at byte 248801 (line 10580, column 29; expected 284427 bytes, actual 284427 bytes) context:   line 10579: "G1 X151.393 Y146.668 Z10"   line 10580: expected "G1 X151.413 Y146.313 Z10.0187"; actual "G1 X151.413 Y146.313 Z10.0188"   line 10581: "G1 X151.568 Y145.992 Z10.0375" |
| PASS | Flashforge/Flashforge Guider4 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.6 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 0.8 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.25 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.4 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 HF nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.6 nozzle |  |
| PASS | Flashforge/Flashforge Guider4 Pro 0.8 HF nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost 6 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Ghost7 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear Reborn3 0.4 nozzle |  |
| PASS | FlyingBear/FlyingBear S1 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-5 0.6 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.4 nozzle |  |
| PASS | Folgertech/Folgertech FT-6 0.6 nozzle |  |
| PASS | Folgertech/Folgertech i3 0.4 nozzle |  |
| PASS | Folgertech/Folgertech i3 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A10 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A10 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A20 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A20 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A20 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech A30 Pro 0.8 nozzle |  |
| PASS | Geeetech/Geeetech A30 T 0.4 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.2 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.4 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.6 nozzle |  |
| PASS | Geeetech/Geeetech M1 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar M 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Max 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar Pro 0.8 nozzle |  |
| DIVERGENT | Geeetech/Geeetech Mizar S 0.2 nozzle | first difference at byte 54428 (line 2284, column 1; expected 316240 bytes, actual 316240 bytes) context:   line 2283: "G1 X125.256 Y128.005 E.00324"   line 2284: expected "M73 P19 R13"; actual "G1 X125.7 Y126.995 E.00893"   line 2285: expected "G1 X125.7 Y126.995 E.00893"; actual "M73 P19 R13" |
| PASS | Geeetech/Geeetech Mizar S 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Mizar S 0.8 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.2 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.4 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.6 nozzle |  |
| PASS | Geeetech/Geeetech Thunder 0.8 nozzle |  |
| DIVERGENT | Ginger Additive/Ginger G1 1.2 nozzle | first difference at byte 597 (line 19, column 10; expected 39732 bytes, actual 39519 bytes) context:   line 18: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=500,500 POLYGON=[[495,495],[505,495],[505,505],[495,505],[495,495]]"   line 19: expected "M73 P0 R10"; actual "M73 P0 R13"   line 20: ";TYPE:Custom" |
| PASS | Ginger Additive/Ginger G1 3.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 5.0 nozzle |  |
| PASS | Ginger Additive/Ginger G1 8.0 nozzle |  |
| PASS | InfiMech/InfiMech EX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech EX+APS 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX 0.4 nozzle |  |
| PASS | InfiMech/InfiMech TX HSN 0.4 nozzle |  |
| PASS | Kingroon/Kingroon KLP1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S 3.0 0.4 nozzle | first difference at byte 1703 (line 76, column 9; expected 92286 bytes, actual 89403 bytes) context:   line 75: "G1 X96.258 Y83.173 E.07012"   line 76: expected "M73 P7 R2"; actual "M73 P7 R3"   line 77: "G1 X97.072 Y84.208 E.07012" |
| PASS | Kingroon/Kingroon KP3S PRO S1 0.4 nozzle |  |
| DIVERGENT | Kingroon/Kingroon KP3S PRO V2 0.4 nozzle | first difference at byte 940 (line 39, column 3; expected 103623 bytes, actual 103327 bytes) context:   line 38: ""   line 39: expected "G10 ; retract"; actual "G1 E-.8 F2700"   line 40: ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Kingroon/Kingroon KP3S V1 0.4 nozzle | first difference at byte 1513 (line 56, column 21; expected 107950 bytes, actual 107944 bytes) context:   line 55: "SET_VELOCITY_LIMIT ACCEL=10000"   line 56: expected "G1 X83.148 Y83.213 F9000"; actual "G1 X83.148 Y83.213 F12000"   line 57: expected "G1 Z.25 F12000"; actual "G1 Z.25" |
| PASS | LH/LH Stinger 0.4 nozzle |  |
| DIVERGENT | LH/LH Stinger MMU 0.4 nozzle | first difference at byte 627 (line 23, column 1; expected 135321 bytes, actual 135322 bytes) context:   line 22: ";TYPE:Custom"   line 23: expected "_SP_PRINT_START LANE=0 TEMP=230"; actual " _SP_PRINT_START LANE=0 TEMP=230"   line 24: "" |
| DIVERGENT | LONGER/LONGER LK10 (0.2 nozzle) | first difference at byte 241 (line 10, column 44; expected 191970 bytes, actual 191970 bytes) context:   line 9: ""   line 10: expected "; external perimeters extrusion width = 0.23mm"; actual "; external perimeters extrusion width = 0.22mm"   line 11: "; perimeters extrusion width = 0.25mm" |
| PASS | LONGER/LONGER LK10 (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 (0.8 nozzle) | first difference at byte 51264 (line 2164, column 29; expected 103095 bytes, actual 103095 bytes) context:   line 2163: "G1 X110.137 Y110.137 E-.05789"   line 2164: expected "G1 X110.944 Y110.137 E-.24211"; actual "G1 X110.944 Y110.137 E-.24212"   line 2165: ";WIPE_END" |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.2 nozzle) | first difference at byte 241 (line 10, column 44; expected 191918 bytes, actual 191918 bytes) context:   line 9: ""   line 10: expected "; external perimeters extrusion width = 0.23mm"; actual "; external perimeters extrusion width = 0.22mm"   line 11: "; perimeters extrusion width = 0.25mm" |
| PASS | LONGER/LONGER LK10 Plus (0.4 nozzle) |  |
| PASS | LONGER/LONGER LK10 Plus (0.6 nozzle) |  |
| DIVERGENT | LONGER/LONGER LK10 Plus (0.8 nozzle) | first difference at byte 51208 (line 2164, column 29; expected 103059 bytes, actual 103059 bytes) context:   line 2163: "G1 X157.637 Y157.637 E-.05789"   line 2164: expected "G1 X158.444 Y157.637 E-.24211"; actual "G1 X158.444 Y157.637 E-.24212"   line 2165: ";WIPE_END" |
| PASS | Lulzbot/Lulzbot Taz 4 or 5 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz 6 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro Dual 0.5 nozzle |  |
| PASS | Lulzbot/Lulzbot Taz Pro S 0.5 nozzle |  |
| PASS | M3D/M3D Enabler D8500 MM |  |
| PASS | MagicMaker/MM BoneKing 0.4 nozzle |  |
| PASS | MagicMaker/MM hj SK 0.4 nozzle |  |
| PASS | MagicMaker/MM hqs SF 0.4 nozzle |  |
| PASS | MagicMaker/MM hqs hj 0.4 nozzle |  |
| PASS | MagicMaker/MM slb 0.4 nozzle |  |
| DIVERGENT | Mellow/M1 0.2 nozzle | first difference at byte 375170 (line 15639, column 62; expected 395090 bytes, actual 395090 bytes) context:   line 15638: "; estimated printing time (normal mode) = 10m 11s"   line 15639: expected "; estimated first layer printing time (normal mode) = 0.438319s"; actual "; estimated first layer printing time (normal mode) = 0.438318s"   line 15640: "" |
| PASS | Mellow/M1 0.4 nozzle |  |
| PASS | Mellow/M1 0.6 nozzle |  |
| PASS | Mellow/M1 0.8 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.2 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.4 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.6 nozzle |  |
| PASS | OpenEYE/OpenEYE Peacock V2 0.8 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.2 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.4 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.6 nozzle |  |
| PASS | OrcaArena/Orca Arena X1 Carbon 0.8 nozzle |  |
| DIVERGENT | Peopoly/Peopoly Magneto X 0.4 nozzle | first difference at byte 185662 (line 6677, column 62; expected 206009 bytes, actual 206009 bytes) context:   line 6676: "; estimated printing time (normal mode) = 7m 15s"   line 6677: expected "; estimated first layer printing time (normal mode) = 0.874670s"; actual "; estimated first layer printing time (normal mode) = 0.874671s"   line 6678: "" |
| PASS | Peopoly/Peopoly Magneto X 0.6 nozzle |  |
| PASS | Peopoly/Peopoly Magneto X 0.8 nozzle |  |
| PASS | Phrozen/Phrozen Arco 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.2 nozzle |  |
| PASS | Positron3D/The Positron 0.4 nozzle |  |
| PASS | Positron3D/The Positron 0.6 nozzle |  |
| PASS | Positron3D/The Positron 0.8 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.25 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One 0.4 nozzle | first difference at byte 6667 (line 300, column 1; expected 96233 bytes, actual 96233 bytes) context:   line 299: "G1 X121.021 Y111.083 E.24231"   line 300: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 301: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One HF 0.4 nozzle | first difference at byte 6631 (line 297, column 1; expected 95737 bytes, actual 95737 bytes) context:   line 296: "G1 X121.021 Y111.083 E.24231"   line 297: expected "M73 P86 R4"; actual "G1 X121.021 Y110.507 E.01949"   line 298: expected "G1 X121.021 Y110.507 E.01949"; actual "M73 P86 R4" |
| PASS | Prusa/Prusa CORE One HF 0.5 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.6 nozzle |  |
| PASS | Prusa/Prusa CORE One HF 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa CORE One L 0.4 nozzle | first difference at byte 7475 (line 343, column 6; expected 96077 bytes, actual 96077 bytes) context:   line 342: ";WIDTH:0.45"   line 343: expected "G1 F1912"; actual "G1 F1899"   line 344: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 90693 bytes, actual 90702 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L 0.6 nozzle | first difference at byte 2106 (line 106, column 7; expected 75897 bytes, actual 75886 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P89 R3"; actual "M73 P88 R3"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51843 bytes, actual 51842 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.4 nozzle | first difference at byte 7439 (line 340, column 6; expected 95609 bytes, actual 95609 bytes) context:   line 339: ";WIDTH:0.45"   line 340: expected "G1 F1912"; actual "G1 F1899"   line 341: "G1 X145.675 Y154.325 E.29279" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.5 nozzle | first difference at byte 514 (line 21, column 9; expected 89688 bytes, actual 89697 bytes) context:   line 20: "M486 S-1"   line 21: expected "M73 P0 R29"; actual "M73 P0 R30"   line 22: "M201 X10000 Y10000 Z400 E5000" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.6 nozzle | first difference at byte 2107 (line 106, column 7; expected 67236 bytes, actual 67235 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P91 R2"; actual "M73 P90 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| DIVERGENT | Prusa/Prusa CORE One L HF 0.8 nozzle | first difference at byte 2106 (line 106, column 7; expected 51902 bytes, actual 51901 bytes) context:   line 105: "G1 E2 F2400 ; deretraction after the initial one"   line 106: expected "M73 P93 R1"; actual "M73 P92 R2"   line 107: "G0 E5 X235 Z0.2 F500 ; purge" |
| PASS | Prusa/Prusa MINI 0.25 nozzle |  |
| PASS | Prusa/Prusa MINI 0.4 nozzle |  |
| DIVERGENT | Prusa/Prusa MINI 0.6 nozzle | first difference at byte 21606 (line 1187, column 2; expected 89826 bytes, actual 89826 bytes) context:   line 1186: "G1 E3.2 F1800"   line 1187: expected "M73 P57 R4"; actual "M205 X8 Y8"   line 1188: expected "M205 X8 Y8"; actual ";TYPE:Inner wall" |
| PASS | Prusa/Prusa MINI 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MINIIS 0.25 nozzle | first difference at byte 21777 (line 975, column 1; expected 215474 bytes, actual 215473 bytes) context:   line 974: "G1 X89.28 Y94.131 E.00421"   line 975: expected "M73 P29 R13"; actual "G1 X94.131 Y89.28 E.0836"   line 976: expected "G1 X94.131 Y89.28 E.0836"; actual "M73 P29 R13" |
| DIVERGENT | Prusa/Prusa MINIIS 0.4 nozzle | first difference at byte 2782 (line 133, column 1; expected 107296 bytes, actual 107309 bytes) context:   line 132: "G1 X92.051 Y86.75 E.06609"   line 133: expected "M73 P35 R8"; actual "G1 X91.388 Y86.75 E.02581"   line 134: expected "G1 X91.388 Y86.75 E.02581"; actual "M73 P35 R8" |
| DIVERGENT | Prusa/Prusa MINIIS 0.6 nozzle | first difference at byte 18103 (line 956, column 13; expected 78246 bytes, actual 78246 bytes) context:   line 955: ";3.25"   line 956: expected "M74 W0.250376"; actual "M74 W0.250375"   line 957: "" |
| PASS | Prusa/Prusa MINIIS 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3.5 0.25 nozzle | first difference at byte 156733 (line 7130, column 13; expected 226919 bytes, actual 226919 bytes) context:   line 7129: ";8.43"   line 7130: expected "M74 W0.480576"; actual "M74 W0.480575"   line 7131: "" |
| DIVERGENT | Prusa/Prusa MK3.5 0.4 nozzle | first difference at byte 4896 (line 228, column 1; expected 111095 bytes, actual 111095 bytes) context:   line 227: "G1 X129.775 Y100.225 E.32326"   line 228: expected "M73 P72 R8"; actual "G1 X129.775 Y109.735 E.3219"   line 229: expected "G1 X129.775 Y109.735 E.3219"; actual "M73 P72 R8" |
| PASS | Prusa/Prusa MK3.5 0.6 nozzle |  |
| PASS | Prusa/Prusa MK3.5 0.8 nozzle |  |
| PASS | Prusa/Prusa MK3S 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.4 nozzle | first difference at byte 55392 (line 2695, column 1; expected 95415 bytes, actual 95415 bytes) context:   line 2694: "G1 X129.368 Y109.328 E.28846"   line 2695: expected "M73 P85 R1"; actual "; stop printing object cube10.stl id:0 copy 0"   line 2696: expected "; stop printing object cube10.stl id:0 copy 0"; actual ";LAYER_CHANGE" |
| PASS | Prusa/Prusa MK3S 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK3S 0.8 nozzle | first difference at byte 3732 (line 178, column 1; expected 64861 bytes, actual 64861 bytes) context:   line 177: "G1 X120 Y97.582 E.10208"   line 178: expected "G1 X130 Y97.582 E1.02139"; actual "M73 P6 R4"   line 179: expected "M73 P6 R4"; actual "G1 X130 Y97.582 E1.02139" |
| DIVERGENT | Prusa/Prusa MK4 0.25 nozzle | first difference at byte 276271 (line 12449, column 13; expected 312715 bytes, actual 312715 bytes) context:   line 12448: "M201 X3999.39 Y3999.39"   line 12449: expected "M74 W0.573554"; actual "M74 W0.573553"   line 12450: "G1 E-.65 F1800" |
| PASS | Prusa/Prusa MK4 0.4 nozzle |  |
| PASS | Prusa/Prusa MK4 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4 0.8 nozzle | first difference at byte 17928 (line 978, column 13; expected 67355 bytes, actual 67355 bytes) context:   line 977: "M201 X3999.58 Y3999.58"   line 978: expected "M74 W0.387867"; actual "M74 W0.387866"   line 979: "G1 E-.56 F1800" |
| PASS | Prusa/Prusa MK4S 0.25 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.3 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S 0.4 nozzle | first difference at byte 4519 (line 213, column 2; expected 92830 bytes, actual 92830 bytes) context:   line 212: "G1 E-.7 F2100"   line 213: expected "M73 P84 R4"; actual "M486 S0"   line 214: expected "M486 S0"; actual "G1 X129.325 Y109.325 Z.6 F18000" |
| DIVERGENT | Prusa/Prusa MK4S 0.5 nozzle | first difference at byte 44247 (line 2205, column 13; expected 97391 bytes, actual 97391 bytes) context:   line 2204: ";6.6"   line 2205: expected "M74 W0.459083"; actual "M74 W0.459082"   line 2206: "" |
| PASS | Prusa/Prusa MK4S 0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S 0.8 nozzle |  |
| DIVERGENT | Prusa/Prusa MK4S HF0.4 nozzle | first difference at byte 7240 (line 326, column 7; expected 92234 bytes, actual 92234 bytes) context:   line 325: ";WIDTH:0.45"   line 326: expected "G1 F2560"; actual "G1 F2544"   line 327: "G1 X120.675 Y109.325 E.29279" |
| PASS | Prusa/Prusa MK4S HF0.5 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.6 nozzle |  |
| PASS | Prusa/Prusa MK4S HF0.8 nozzle |  |
| PASS | Prusa/Prusa XL 0.25 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.3 nozzle | first difference at byte 2216 (line 81, column 1; expected 125488 bytes, actual 125536 bytes) context:   line 80: "G0 X73 Z0.05 F8000 ; wipe, move close to the bed"   line 81: expected "M73 P77 R9"; actual "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"   line 82: expected "G0 X76 Z0.2 F8000 ; wipe, move quickly away from the bed"; actual "G92 E0 ; reset extruder position" |
| DIVERGENT | Prusa/Prusa XL 0.4 nozzle | first difference at byte 3176 (line 135, column 1; expected 93107 bytes, actual 93155 bytes) context:   line 134: "G1 E.8 F1800"   line 135: expected "M73 P83 R6"; actual ";TYPE:Bottom surface"   line 136: expected ";TYPE:Bottom surface"; actual ";WIDTH:0.50675" |
| PASS | Prusa/Prusa XL 0.5 nozzle |  |
| PASS | Prusa/Prusa XL 0.6 nozzle |  |
| DIVERGENT | Prusa/Prusa XL 0.8 nozzle | first difference at byte 4422 (line 198, column 22; expected 53778 bytes, actual 53824 bytes) context:   line 197: ";WIPE_END"   line 198: expected "G1 X181.559 Y183.324 F24000"; actual "G1 X181.559 Y183.324 Z1 F24000"   line 199: expected "G1 E.6 F1800"; actual "G1 Z.6" |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.25 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-kdHAnE") |
| DIVERGENT | Prusa/Prusa XL 5T 0.3 nozzle | first difference at byte 3004 (line 151, column 1; expected 136093 bytes, actual 136154 bytes) context:   line 150: ""   line 151: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F2100"   line 152: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| DIVERGENT | Prusa/Prusa XL 5T 0.4 nozzle | first difference at byte 2982 (line 149, column 1; expected 103701 bytes, actual 103762 bytes) context:   line 148: ""   line 149: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F2100"   line 150: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.5 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-thS9Y7") |
| ORCA_ERROR | Prusa/Prusa XL 5T 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-Dxbv6A") |
| DIVERGENT | Prusa/Prusa XL 5T 0.8 nozzle | first difference at byte 2982 (line 149, column 1; expected 64354 bytes, actual 64413 bytes) context:   line 148: ""   line 149: expected ";AFTER_LAYER_CHANGE"; actual "G1 E-.8 F2100"   line 150: expected ";0.2"; actual ";AFTER_LAYER_CHANGE" |
| PASS | Qidi/Qidi Q1 Pro 0.2 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.6 nozzle |  |
| PASS | Qidi/Qidi Q1 Pro 0.8 nozzle |  |
| PASS | Qidi/Qidi Q2 0.2 nozzle |  |
| PASS | Qidi/Qidi Q2 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2 0.8 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.2 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.4 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.6 nozzle |  |
| PASS | Qidi/Qidi Q2C 0.8 nozzle |  |
| PASS | Qidi/Qidi X-CF Pro 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 0.4 nozzle |  |
| DIVERGENT | Qidi/Qidi X-Max 3 0.2 nozzle | first difference at byte 23738 (line 930, column 1; expected 299355 bytes, actual 299357 bytes) context:   line 929: "G1 X166.626 Y163.985 E.00227"   line 930: expected "M73 P28 R13"; actual "G1 X161.015 Y158.374 E.06418"   line 931: expected "G1 X161.015 Y158.374 E.06418"; actual "G1 X160.734 Y158.374 E.00227" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.4 nozzle | first difference at byte 15161 (line 656, column 1; expected 121136 bytes, actual 121138 bytes) context:   line 655: "G1 X167.29 Y157.71 E.29437"   line 656: expected "M73 P52 R5"; actual "G1 X167.29 Y167.23 E.29252"   line 657: expected "G1 X167.29 Y167.23 E.29252"; actual "M73 P52 R5" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.6 nozzle | first difference at byte 3973 (line 173, column 1; expected 75688 bytes, actual 75690 bytes) context:   line 172: "G1 Z.6"   line 173: expected "M73 P58 R3"; actual "G1 E1.4 F1800"   line 174: expected "G1 E1.4 F1800"; actual "M73 P58 R3" |
| DIVERGENT | Qidi/Qidi X-Max 3 0.8 nozzle | first difference at byte 2982 (line 134, column 1; expected 69152 bytes, actual 69154 bytes) context:   line 133: "G1 X161.32 Y165.655 E.34889"   line 134: expected "M73 P63 R2"; actual "G1 X160.236 Y165.655 E.13536"   line 135: expected "G1 X160.236 Y165.655 E.13536"; actual "M73 P63 R2" |
| PASS | Qidi/Qidi X-Max 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Max 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 3 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Plus 4 0.8 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.2 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.4 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.6 nozzle |  |
| PASS | Qidi/Qidi X-Smart 3 0.8 nozzle |  |
| DIVERGENT | RH3D/E3NG v1.2S - 0.2 nozzle | first difference at byte 13777 (line 523, column 11; expected 291096 bytes, actual 291053 bytes) context:   line 522: "G1 X121 Y105.338 E.10775"   line 523: expected "G1 X123.081 Y105.734 E.02283"; actual "G1 X123.082 Y105.734 E.02283"   line 524: "G1 X124.872 Y106.868 E.02283" |
| DIVERGENT | RH3D/E3NG v1.2S - 0.3 nozzle | first difference at byte 312 (line 12, column 31; expected 165499 bytes, actual 165455 bytes) context:   line 11: "; perimeters extrusion width = 0.33mm"   line 12: expected "; infill extrusion width = 0.35mm"; actual "; infill extrusion width = 0.34mm"   line 13: "; solid infill extrusion width = 0.33mm" |
| DIVERGENT | RH3D/E3NG v1.2S - 0.4 nozzle | first difference at byte 9145 (line 353, column 1; expected 120711 bytes, actual 120678 bytes) context:   line 352: "G1 X121 Y108.404 E.30376"   line 353: expected "G1 X122.372 Y108.797 E.04335"; actual "M73 P6 R5"   line 354: expected "M73 P6 R5"; actual "G1 X122.372 Y108.797 E.04335" |
| DIVERGENT | RH3D/E3NG v1.2S - 0.5 nozzle | first difference at byte 6247 (line 241, column 1; expected 111310 bytes, actual 111267 bytes) context:   line 240: "G1 X107.113 Y109.203 E.07137"   line 241: expected "G1 X107.911 Y108.153 E.05116"; actual "M73 P4 R5"   line 242: expected "M73 P4 R5"; actual "G1 X107.911 Y108.153 E.05116" |
| DIVERGENT | RH3D/E3NG v1.2S - 0.6 nozzle | first difference at byte 5646 (line 220, column 4; expected 85574 bytes, actual 85529 bytes) context:   line 219: "; printing object cube10.stl id:0 copy 0"   line 220: expected "G1 E-.48 F2400"; actual "G1 X120.27 Y120.05 F24000"   line 221: expected ";WIPE_START"; actual "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0" |
| DIVERGENT | Raise3D/Raise3D Pro3 0.4 nozzle (Dual) | first difference at byte 8100 (line 363, column 11; expected 110489 bytes, actual 110489 bytes) context:   line 362: ""   line 363: expected "M106 P2 S150"; actual "M106 P2 S100"   line 364: "" |
| DIVERGENT | Raise3D/Raise3D Pro3 0.4 nozzle (Left) | first difference at byte 7908 (line 355, column 11; expected 110669 bytes, actual 110669 bytes) context:   line 354: ""   line 355: expected "M106 P2 S150"; actual "M106 P2 S100"   line 356: "" |
| DIVERGENT | Raise3D/Raise3D Pro3 0.4 nozzle (Right) | first difference at byte 7912 (line 355, column 11; expected 110707 bytes, actual 110707 bytes) context:   line 354: ""   line 355: expected "M106 P2 S150"; actual "M106 P2 S100"   line 356: "" |
| DIVERGENT | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Dual) | first difference at byte 8093 (line 362, column 11; expected 110522 bytes, actual 110522 bytes) context:   line 361: ""   line 362: expected "M106 P2 S150"; actual "M106 P2 S100"   line 363: "" |
| DIVERGENT | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Left) | first difference at byte 7901 (line 354, column 11; expected 110702 bytes, actual 110702 bytes) context:   line 353: ""   line 354: expected "M106 P2 S150"; actual "M106 P2 S100"   line 355: "" |
| DIVERGENT | Raise3D/Raise3D Pro3 Plus 0.4 nozzle (Right) | first difference at byte 7905 (line 354, column 11; expected 110740 bytes, actual 110740 bytes) context:   line 353: ""   line 354: expected "M106 P2 S150"; actual "M106 P2 S100"   line 355: "" |
| PASS | Ratrig/RatRig V-Cast 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Cast 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 200 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 3 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 300 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 300 0.8 nozzle | Ratrig/RatRig V-Core 4 300 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 400 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 400 0.8 nozzle | Ratrig/RatRig V-Core 4 400 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 500 0.6 nozzle |  |
| VENDOR_INCOMPLETE | Ratrig/RatRig V-Core 4 500 0.8 nozzle | Ratrig/RatRig V-Core 4 500 0.8 nozzle process: no compatible preset |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.5 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 300 0.6 nozzle | first difference at byte 82618 (line 3381, column 61; expected 102863 bytes, actual 102863 bytes) context:   line 3380: "; estimated printing time (normal mode) = 2m 45s"   line 3381: expected "; estimated first layer printing time (normal mode) = 0.455069s"; actual "; estimated first layer printing time (normal mode) = 0.455037s"   line 3382: "" |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 300 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.6 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 400 0.8 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.4 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.5 nozzle |  |
| PASS | Ratrig/RatRig V-Core 4 HYBRID 500 0.6 nozzle |  |
| DIVERGENT | Ratrig/RatRig V-Core 4 HYBRID 500 0.8 nozzle | first difference at byte 62871 (line 2488, column 61; expected 83126 bytes, actual 83126 bytes) context:   line 2487: "; estimated printing time (normal mode) = 2m 48s"   line 2488: expected "; estimated first layer printing time (normal mode) = 0.736964s"; actual "; estimated first layer printing time (normal mode) = 0.736947s"   line 2489: "" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.4 nozzle | first difference at byte 4751 (line 177, column 1; expected 119592 bytes, actual 119434 bytes) context:   line 176: "G1 X153.388 Y151.507 E.37511"   line 177: expected "G1 X153.388 Y152.134 E.03398"; actual "M73 P5 R2"   line 178: expected "M73 P5 R2"; actual "G1 X153.388 Y152.134 E.03398" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.5 nozzle | first difference at byte 5550 (line 213, column 22; expected 111488 bytes, actual 111341 bytes) context:   line 212: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 213: expected "G1 X153.698 Y153.698 F36000"; actual "G1 X153.698 Y153.698 Z.4 F36000"   line 214: expected "G1 Z.4"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.6 nozzle | first difference at byte 5534 (line 215, column 22; expected 104653 bytes, actual 104495 bytes) context:   line 214: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 215: expected "G1 X153.563 Y153.563 F30000"; actual "G1 X153.563 Y153.563 Z.4 F30000"   line 216: expected "G1 Z.4"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 0.8 nozzle | first difference at byte 3015 (line 122, column 1; expected 84753 bytes, actual 84650 bytes) context:   line 121: "G1 Z.45"   line 122: expected "G1 E.8 F7200"; actual "M73 P3 R2"   line 123: expected "M73 P3 R2"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.4 nozzle | first difference at byte 5375 (line 198, column 1; expected 113834 bytes, actual 113676 bytes) context:   line 197: "G1 E-.8 F7200"   line 198: expected "EXCLUDE_OBJECT_END NAME=cube10.stl_id_0_copy_0"; actual "M73 P5 R3"   line 199: expected "TIMELAPSE_TAKE_FRAME"; actual "EXCLUDE_OBJECT_END NAME=cube10.stl_id_0_copy_0" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.5 nozzle | first difference at byte 5269 (line 206, column 1; expected 106149 bytes, actual 105994 bytes) context:   line 205: "G1 E.8 F7200"   line 206: expected "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"; actual "M73 P4 R3"   line 207: expected ";TYPE:Inner wall"; actual "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.6 nozzle | first difference at byte 2691 (line 102, column 1; expected 99086 bytes, actual 98942 bytes) context:   line 101: "G1 X156.873 Y165.208 E.18095"   line 102: expected "G1 X155 Y165.379 E.11842"; actual "M73 P2 R3"   line 103: expected "M73 P2 R3"; actual "G1 X155 Y165.379 E.11842" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 COPY MODE 0.8 nozzle | first difference at byte 3029 (line 124, column 1; expected 84807 bytes, actual 84704 bytes) context:   line 123: "G1 Z.45"   line 124: expected "G1 E.8 F7200"; actual "M73 P3 R2"   line 125: expected "M73 P3 R2"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.4 nozzle | first difference at byte 5816 (line 216, column 22; expected 113842 bytes, actual 113684 bytes) context:   line 215: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 216: expected "G1 X138.918 Y153.918 F30000"; actual "G1 X138.918 Y153.918 Z.5 F30000"   line 217: expected "G1 Z.5"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.5 nozzle | first difference at byte 6066 (line 236, column 1; expected 106157 bytes, actual 106013 bytes) context:   line 235: "G1 E.8 F7200"   line 236: expected ";TYPE:Internal solid infill"; actual "M73 P5 R3"   line 237: expected ";WIDTH:0.456051"; actual ";TYPE:Internal solid infill" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.6 nozzle | first difference at byte 7551 (line 296, column 22; expected 99094 bytes, actual 98950 bytes) context:   line 295: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 296: expected "G1 X138.563 Y153.563 F30000"; actual "G1 X138.563 Y153.563 Z.6 F30000"   line 297: expected "G1 Z.6"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 300 MIRROR MODE 0.8 nozzle | first difference at byte 3044 (line 125, column 1; expected 84815 bytes, actual 84712 bytes) context:   line 124: "G1 E.8 F7200"   line 125: expected "SET_VELOCITY_LIMIT ACCEL=20000 ACCEL_TO_DECEL=10000"; actual "M73 P3 R2"   line 126: expected ";TYPE:Inner wall"; actual "SET_VELOCITY_LIMIT ACCEL=20000 ACCEL_TO_DECEL=10000" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 0.4 nozzle | first difference at byte 3031 (line 112, column 1; expected 119592 bytes, actual 119434 bytes) context:   line 111: "G1 X186.343 Y189.572 E.15186"   line 112: expected "G1 X188.05 Y187.546 E.14106"; actual "M73 P3 R2"   line 113: expected "M73 P3 R2"; actual "G1 X188.05 Y187.546 E.14106" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 0.5 nozzle | first difference at byte 1718 (line 64, column 1; expected 111488 bytes, actual 111341 bytes) context:   line 63: "G1 X212.573 Y212.958 E.15837"   line 64: expected "G1 X210.175 Y214.69 E.15837"; actual "M73 P1 R2"   line 65: expected "M73 P1 R2"; actual "G1 X210.175 Y214.69 E.15837" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 0.6 nozzle | first difference at byte 2982 (line 110, column 1; expected 104653 bytes, actual 104495 bytes) context:   line 109: "G1 X184.621 Y195 E.61692"   line 110: expected "G1 X185.02 Y192.153 E.17734"; actual "M73 P3 R2"   line 111: expected "M73 P3 R2"; actual "G1 X185.02 Y192.153 E.17734" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 0.8 nozzle | first difference at byte 5599 (line 219, column 22; expected 84753 bytes, actual 84650 bytes) context:   line 218: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 219: expected "G1 X203.229 Y203.229 F48000"; actual "G1 X203.229 Y203.229 Z.7 F48000"   line 220: expected "G1 Z.7"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.4 nozzle | first difference at byte 5257 (line 194, column 1; expected 113827 bytes, actual 113680 bytes) context:   line 193: "G1 X197.744 Y203.388 E.08853"   line 194: expected "G1 X197.117 Y203.388 E.03468"; actual "M73 P5 R3"   line 195: expected "M73 P5 R3"; actual "G1 X197.117 Y203.388 E.03468" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.5 nozzle | first difference at byte 2708 (line 104, column 1; expected 106164 bytes, actual 106009 bytes) context:   line 103: "G1 X195 Y215.329 E.54636"   line 104: expected "G1 X192.16 Y214.93 E.15667"; actual "M73 P2 R3"   line 105: expected "M73 P2 R3"; actual "G1 X192.16 Y214.93 E.15667" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.6 nozzle | first difference at byte 1787 (line 70, column 1; expected 99090 bytes, actual 98946 bytes) context:   line 69: "G1 X195 Y216.136 E.62951"   line 70: expected "G1 X192.049 Y215.737 E.18747"; actual "M73 P1 R3"   line 71: expected "M73 P1 R3"; actual "G1 X192.049 Y215.737 E.18747" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 COPY MODE 0.8 nozzle | first difference at byte 5613 (line 221, column 22; expected 84811 bytes, actual 84708 bytes) context:   line 220: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 221: expected "G1 X203.229 Y203.229 F48000"; actual "G1 X203.229 Y203.229 Z.7 F48000"   line 222: expected "G1 Z.7"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.4 nozzle | first difference at byte 2711 (line 103, column 1; expected 113835 bytes, actual 113688 bytes) context:   line 102: "G1 X190 Y215.218 E.09459"   line 103: expected "G1 X180 Y215.218 E.54333"; actual "M73 P2 R3"   line 104: expected "M73 P2 R3"; actual "G1 X180 Y215.218 E.54333" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.5 nozzle | first difference at byte 3741 (line 142, column 1; expected 106172 bytes, actual 106017 bytes) context:   line 141: "G1 X189.55 Y204.55 Z.4"   line 142: expected "G1 Z.2"; actual "M73 P3 R3"   line 143: expected "M73 P3 R3"; actual "G1 Z.2" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.6 nozzle | first difference at byte 5692 (line 222, column 1; expected 99098 bytes, actual 98954 bytes) context:   line 221: "G1 X189.12 Y204.06 E.37891"   line 222: expected "SET_VELOCITY_LIMIT ACCEL=4000 ACCEL_TO_DECEL=2000"; actual "M73 P5 R3"   line 223: expected "G1 X189.71 Y204.71 F30000"; actual "SET_VELOCITY_LIMIT ACCEL=4000 ACCEL_TO_DECEL=2000" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 400 MIRROR MODE 0.8 nozzle | first difference at byte 5615 (line 221, column 22; expected 84819 bytes, actual 84716 bytes) context:   line 220: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 221: expected "G1 X188.229 Y203.229 F48000"; actual "G1 X188.229 Y203.229 Z.7 F48000"   line 222: expected "G1 Z.7"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.4 nozzle | first difference at byte 3791 (line 140, column 1; expected 119581 bytes, actual 119434 bytes) context:   line 139: "G1 X254.15 Y254.11 E.43981"   line 140: expected "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"; actual "M73 P4 R2"   line 141: expected "G1 X254.65 Y254.65 F30000"; actual "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.5 nozzle | first difference at byte 5560 (line 214, column 22; expected 111488 bytes, actual 111341 bytes) context:   line 213: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 214: expected "G1 X253.698 Y253.698 F36000"; actual "G1 X253.698 Y253.698 Z.4 F36000"   line 215: expected "G1 Z.4"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.6 nozzle | first difference at byte 5534 (line 215, column 22; expected 104653 bytes, actual 104506 bytes) context:   line 214: "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"   line 215: expected "G1 X253.563 Y253.563 F30000"; actual "G1 X253.563 Y253.563 Z.4 F30000"   line 216: expected "G1 Z.4"; actual "G1 E.8 F7200" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 0.8 nozzle | first difference at byte 4541 (line 180, column 1; expected 84764 bytes, actual 84650 bytes) context:   line 179: "G1 X248.717 Y252.658 E.4034"   line 180: expected "G1 X247.732 Y252.658 E.07128"; actual "M73 P5 R2"   line 181: expected "M73 P5 R2"; actual "G1 X247.732 Y252.658 E.07128" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.4 nozzle | first difference at byte 5170 (line 191, column 1; expected 113827 bytes, actual 113680 bytes) context:   line 190: "G1 X248.371 Y253.388 E.03468"   line 191: expected "G1 X246.612 Y251.629 E.13756"; actual "M73 P5 R3"   line 192: expected "G1 X246.612 Y252.256 E.03468"; actual "G1 X246.612 Y251.629 E.13756" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.5 nozzle | first difference at byte 5958 (line 232, column 1; expected 106164 bytes, actual 106009 bytes) context:   line 231: "G1 X254.74 Y254.69 E.37408"   line 232: expected "SET_VELOCITY_LIMIT ACCEL=15000 ACCEL_TO_DECEL=7500"; actual "M73 P5 R3"   line 233: expected "G1 E-.8 F7200"; actual "SET_VELOCITY_LIMIT ACCEL=15000 ACCEL_TO_DECEL=7500" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.6 nozzle | first difference at byte 2575 (line 98, column 1; expected 99090 bytes, actual 98946 bytes) context:   line 97: "G1 X264.98 Y257.847 E.18095"   line 98: expected "G1 X263.817 Y260.475 E.18095"; actual "M73 P2 R3"   line 99: expected "M73 P2 R3"; actual "G1 X263.817 Y260.475 E.18095" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 COPY MODE 0.8 nozzle | first difference at byte 4555 (line 182, column 1; expected 84822 bytes, actual 84708 bytes) context:   line 181: "G1 X248.717 Y252.658 E.4034"   line 182: expected "G1 X247.732 Y252.658 E.07128"; actual "M73 P5 R2"   line 183: expected "M73 P5 R2"; actual "G1 X247.732 Y252.658 E.07128" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.4 nozzle | first difference at byte 2657 (line 101, column 1; expected 113835 bytes, actual 113688 bytes) context:   line 100: "G1 X244.45 Y264.198 E.15496"   line 101: expected "G1 X241.735 Y265.069 E.15496"; actual "M73 P2 R3"   line 102: expected "M73 P2 R3"; actual "G1 X241.735 Y265.069 E.15496" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.5 nozzle | first difference at byte 7101 (line 275, column 1; expected 106172 bytes, actual 106017 bytes) context:   line 274: "G1 X231.718 Y248.721 E.03024"   line 275: expected "G1 X233.721 Y246.718 E.11942"; actual "M73 P6 R3"   line 276: expected "G1 X233.004 Y246.718 E.03024"; actual "G1 X233.721 Y246.718 E.11942" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.6 nozzle | first difference at byte 3503 (line 132, column 1; expected 99098 bytes, actual 98954 bytes) context:   line 131: "G1 E-.8 F7200"   line 132: expected "G1 Z.4 F30000"; actual "M73 P3 R3"   line 133: expected "G1 X238.7 Y253.7 Z.4"; actual "G1 Z.4 F30000" |
| DIVERGENT | Ratrig/RatRig V-Core 4 IDEX 500 MIRROR MODE 0.8 nozzle | first difference at byte 4557 (line 182, column 1; expected 84830 bytes, actual 84716 bytes) context:   line 181: "G1 X233.717 Y252.658 E.4034"   line 182: expected "G1 X232.732 Y252.658 E.07128"; actual "M73 P5 R2"   line 183: expected "M73 P5 R2"; actual "G1 X232.732 Y252.658 E.07128" |
| PASS | Ratrig/RatRig V-Minion 0.4 nozzle |  |
| DIVERGENT | RolohaunDesign/Rolohaun Delta Flyer Refit 0.4 nozzle | first difference at byte 86654 (line 3704, column 16; expected 126094 bytes, actual 126094 bytes) context:   line 3703: "G1 X-1.055 Y6.015 E.00391"   line 3704: expected "G1 X5.877 Y6.014 E.19257"; actual "G1 X5.877 Y6.013 E.19257"   line 3705: "G1 X5.986 Y5.986 E.00312" |
| PASS | RolohaunDesign/Rook MK1 LDO 0.2 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.4 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.6 nozzle |  |
| PASS | RolohaunDesign/Rook MK1 LDO 0.8 nozzle |  |
| PASS | SecKit/SecKit Go3 0.4 nozzle |  |
| PASS | SecKit/SecKit SK-Tank 0.4 nozzle |  |
| DIVERGENT | SeeMeCNC/SeeMeCNC Artemis 0.4 nozzle | first difference at byte 1498 (line 47, column 9; expected 150664 bytes, actual 150695 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P1 R8"; actual "M73 P1 R7"   line 48: "G3 X50 Y-129.9 R139.2 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC Artemis 0.5 nozzle | first difference at byte 511 (line 20, column 9; expected 112668 bytes, actual 112689 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R6"; actual "M73 P0 R5"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC Artemis 0.7 nozzle | first difference at byte 1288 (line 42, column 1; expected 77483 bytes, actual 77524 bytes) context:   line 41: "G1 E160.5 F3000  ; Retract 160mm"   line 42: expected "G92 E0"; actual "M73 P0 R3"   line 43: expected "; --- PURGE LINE (curved arc near front edge, 290mm bed) ---"; actual "G92 E0" |
| DIVERGENT | SeeMeCNC/SeeMeCNC Artemis 1.0 nozzle | first difference at byte 1573 (line 49, column 1; expected 60435 bytes, actual 60475 bytes) context:   line 48: "G3 X50 Y-129.9 R139.2 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion"   line 49: expected "M73 P10 R2"; actual "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"   line 50: expected "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"; actual "G1 X65 Y-119.9 Z0.2 F6000     ; Wipe move at Z0.2" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.4 nozzle | first difference at byte 1504 (line 47, column 9; expected 150680 bytes, actual 150711 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P1 R8"; actual "M73 P1 R7"   line 48: "G3 X50 Y-135.0 R144.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.5 nozzle | first difference at byte 511 (line 20, column 9; expected 112684 bytes, actual 112705 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R6"; actual "M73 P0 R5"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 0.7 nozzle | first difference at byte 1294 (line 42, column 1; expected 77500 bytes, actual 77540 bytes) context:   line 41: "G1 E160.5 F3000  ; Retract 160mm"   line 42: expected "G92 E0"; actual "M73 P0 R3"   line 43: expected "; --- PURGE LINE (curved arc near front edge, 300mm bed) ---"; actual "G92 E0" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 300 1.0 nozzle | first difference at byte 1579 (line 49, column 1; expected 60451 bytes, actual 60491 bytes) context:   line 48: "G3 X50 Y-135.0 R144.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion"   line 49: expected "M73 P10 R2"; actual "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"   line 50: expected "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"; actual "G1 X65 Y-125.0 Z0.2 F6000     ; Wipe move at Z0.2" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.4 nozzle | first difference at byte 1508 (line 47, column 9; expected 150729 bytes, actual 150738 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R8"; actual "M73 P2 R7"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.5 nozzle | first difference at byte 1508 (line 47, column 9; expected 112723 bytes, actual 112753 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R6"; actual "M73 P2 R5"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 0.7 nozzle | first difference at byte 1508 (line 47, column 9; expected 77548 bytes, actual 77567 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P4 R4"; actual "M73 P4 R3"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0505 1.0 nozzle | first difference at byte 510 (line 20, column 9; expected 60488 bytes, actual 60530 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R3"; actual "M73 P0 R2"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.4 nozzle | first difference at byte 1508 (line 47, column 9; expected 150720 bytes, actual 150740 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R8"; actual "M73 P2 R7"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.5 nozzle | first difference at byte 1508 (line 47, column 9; expected 112725 bytes, actual 112755 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R6"; actual "M73 P2 R5"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 0.7 nozzle | first difference at byte 1508 (line 47, column 9; expected 77540 bytes, actual 77569 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P4 R4"; actual "M73 P4 R3"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0510 1.0 nozzle | first difference at byte 510 (line 20, column 9; expected 60468 bytes, actual 60532 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R3"; actual "M73 P0 R2"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.4 nozzle | first difference at byte 1508 (line 47, column 9; expected 150720 bytes, actual 150740 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R8"; actual "M73 P2 R7"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.5 nozzle | first difference at byte 1508 (line 47, column 9; expected 112725 bytes, actual 112755 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P2 R6"; actual "M73 P2 R5"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 0.7 nozzle | first difference at byte 1508 (line 47, column 9; expected 77540 bytes, actual 77569 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P4 R4"; actual "M73 P4 R3"   line 48: "G3 X50 Y-234.7 R240.0 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC BOSSdelta 500 0521 1.0 nozzle | first difference at byte 510 (line 20, column 9; expected 60468 bytes, actual 60532 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R3"; actual "M73 P0 R2"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.4 nozzle | first difference at byte 1506 (line 47, column 9; expected 150698 bytes, actual 150729 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P1 R8"; actual "M73 P1 R7"   line 48: "G3 X50 Y-119.6 R129.6 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.5 nozzle | first difference at byte 511 (line 20, column 9; expected 112703 bytes, actual 112724 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R6"; actual "M73 P0 R5"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 0.7 nozzle | first difference at byte 1296 (line 42, column 1; expected 77517 bytes, actual 77547 bytes) context:   line 41: "G1 E160.5 F3000  ; Retract 160mm"   line 42: expected "G92 E0"; actual "M73 P0 R3"   line 43: expected "; --- PURGE LINE (curved arc near front edge, 270mm bed) ---"; actual "G92 E0" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v3.2 1.0 nozzle | first difference at byte 1581 (line 49, column 1; expected 60470 bytes, actual 60510 bytes) context:   line 48: "G3 X50 Y-119.6 R129.6 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion"   line 49: expected "M73 P10 R2"; actual "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"   line 50: expected "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"; actual "G1 X65 Y-109.6 Z0.2 F6000     ; Wipe move at Z0.2" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v4 0.4 nozzle | first difference at byte 1504 (line 47, column 9; expected 150680 bytes, actual 150711 bytes) context:   line 46: "G1 Z0.3 F1000                    ; Drop to prime height"   line 47: expected "M73 P1 R8"; actual "M73 P1 R7"   line 48: "G3 X50 Y-124.8 R134.4 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v4 0.5 nozzle | first difference at byte 511 (line 20, column 9; expected 112684 bytes, actual 112705 bytes) context:   line 19: "M486 S-1"   line 20: expected "M73 P0 R6"; actual "M73 P0 R5"   line 21: "M201 X5000 Y5000 Z5000 E5000" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v4 0.7 nozzle | first difference at byte 1294 (line 42, column 1; expected 77499 bytes, actual 77529 bytes) context:   line 41: "G1 E160.5 F3000  ; Retract 160mm"   line 42: expected "G92 E0"; actual "M73 P0 R3"   line 43: expected "; --- PURGE LINE (curved arc near front edge, 280mm bed) ---"; actual "G92 E0" |
| DIVERGENT | SeeMeCNC/SeeMeCNC RostockMAX v4 1.0 nozzle | first difference at byte 1579 (line 49, column 1; expected 60451 bytes, actual 60491 bytes) context:   line 48: "G3 X50 Y-124.8 R134.4 E40 F600  ; Arc purge, 100mm sweep, heavy extrusion"   line 49: expected "M73 P10 R2"; actual "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"   line 50: expected "G1 E38 F4500                     ; Retract 2mm before wipe (Bowden)"; actual "G1 X65 Y-114.8 Z0.2 F6000     ; Wipe move at Z0.2" |
| PASS | Snapmaker/Snapmaker A250 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A250 QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual BKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 Dual QSKit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QS+B Kit (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker A350 QSKit (0.8 nozzle) |  |
| DIVERGENT | Snapmaker/Snapmaker Artisan (0.2 nozzle) | first difference at byte 13345 (line 589, column 1; expected 188207 bytes, actual 188207 bytes) context:   line 588: "G1 X204.129 Y204.114 E.00282"   line 589: expected "G1 X195.871 Y195.871 E.12255"; actual "M73 P12 R11"   line 590: expected "M73 P12 R11"; actual "G1 X195.871 Y195.871 E.12255" |
| PASS | Snapmaker/Snapmaker Artisan (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker Artisan (0.8 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.2 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.4 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.6 nozzle) |  |
| PASS | Snapmaker/Snapmaker J1 (0.8 nozzle) |  |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.2 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-6twijW") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4 nozzle) | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-ske0uz") |
| ORCA_ERROR | Snapmaker/Snapmaker U1 (0.4+0.6 nozzle) | Export/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-MveBTo") |
| DIVERGENT | Snapmaker/Snapmaker U1 (0.6 nozzle) | first difference at byte 1509 (line 70, column 1; expected 103679 bytes, actual 103700 bytes) context:   line 69: "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"   line 70: expected "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"; actual "G1 E-.5 F1500"   line 71: expected "G1 X139.42 Y139.92 F30000"; actual "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0" |
| DIVERGENT | Snapmaker/Snapmaker U1 (0.8 nozzle) | first difference at byte 1495 (line 69, column 1; expected 66408 bytes, actual 66428 bytes) context:   line 68: "SET_VELOCITY_LIMIT ACCEL=10000 ACCEL_TO_DECEL=5000"   line 69: expected "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0"; actual "G1 E-.5 F1500"   line 70: expected "G1 X139.206 Y139.706 F30000"; actual "EXCLUDE_OBJECT_START NAME=cube10.stl_id_0_copy_0" |
| PASS | Sovol/Sovol SV01 0.4 nozzle |  |
| PASS | Sovol/Sovol SV01 Pro 0.4 nozzle |  |
| PASS | Sovol/Sovol SV02 0.4 nozzle |  |
| PASS | Sovol/Sovol SV05 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV06 0.4 High-Speed nozzle | first difference at byte 1666 (line 90, column 19; expected 101958 bytes, actual 101958 bytes) context:   line 89: "G1 X117.495 Y97.735 E.05224"   line 90: expected "G1 X118.551 Y98.188 E.05224"; actual "G1 X118.551 Y98.189 E.05224"   line 91: "G1 X119.527 Y98.794 E.05224" |
| PASS | Sovol/Sovol SV06 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.2 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.6 nozzle |  |
| PASS | Sovol/Sovol SV06 ACE 0.8 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus 0.4 nozzle |  |
| PASS | Sovol/Sovol SV06 Plus ACE 0.4 nozzle |  |
| PASS | Sovol/Sovol SV07 0.4 nozzle |  |
| DIVERGENT | Sovol/Sovol SV07 Plus 0.4 nozzle | first difference at byte 4032 (line 174, column 1; expected 107311 bytes, actual 107311 bytes) context:   line 173: "G1 X146.375 Y150.578 E5.51963"   line 174: expected "G1 X149.422 Y153.625 E5.67617"; actual "M106 S255"   line 175: expected "G1 X148.896 Y153.625 E5.69525"; actual "G1 X149.422 Y153.625 E5.67617" |
| PASS | Sovol/Sovol SV08 0.2 nozzle |  |
| PASS | Sovol/Sovol SV08 0.4 nozzle |  |
| PASS | Sovol/Sovol SV08 0.6 nozzle |  |
| PASS | Sovol/Sovol SV08 0.8 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.4 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.6 nozzle |  |
| PASS | Sovol/Sovol SV08 MAX 0.8 nozzle |  |
| PASS | Sovol/Sovol Zero 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP300 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP310 Pro 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP400 Pro 0.8 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.4 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.6 nozzle |  |
| PASS | Tiertime/Tiertime UP600 HS 0.8 nozzle |  |
| PASS | Tronxy/Tronxy X5SA 400 0.4 nozzle |  |
| DIVERGENT | TwoTrees/TwoTrees SK1 0.4 nozzle | first difference at byte 1414 (line 60, column 1; expected 101783 bytes, actual 102276 bytes) context:   line 59: "G1 X190 Y12 F6000 ;Wipe"   line 60: expected "M73 P66 R2"; actual "G1 X180 Y8 F6000 ;Wipe"   line 61: expected "G1 X180 Y8 F6000 ;Wipe"; actual "G1 X170 Y12 F6000 ;Wipe" |
| PASS | TwoTrees/TwoTrees SP-5 Klipper 0.4 nozzle |  |
| DIVERGENT | UltiMaker/UltiMaker 2 0.4 nozzle | first difference at byte 467 (line 18, column 9; expected 114516 bytes, actual 114530 bytes) context:   line 17: "; EXECUTABLE_BLOCK_START"   line 18: expected "M73 P0 R5"; actual "M73 P0 R6"   line 19: "M201 X3000 Y3000 Z500 E10000" |
| PASS | Vivedino/Troodon 2.0 Klipper 0.4 nozzle |  |
| PASS | Vivedino/Troodon 2.0 RRF 0.4 nozzle |  |
| DIVERGENT | Volumic/EXO42 (0.4 nozzle) | first difference at byte 40140 (line 1760, column 1; expected 120293 bytes, actual 120292 bytes) context:   line 1759: "G1 X214.328 Y205.672"   line 1760: expected "M73 P38 R3"; actual "G1 X208.497 Y205.672"   line 1761: expected "G1 X208.497 Y205.672"; actual "M73 P38 R3" |
| PASS | Volumic/EXO42 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO42 Stage 2 (0.4 nozzle) |  |
| DIVERGENT | Volumic/EXO65 (0.6 nozzle) | first difference at byte 39837 (line 1917, column 1; expected 115024 bytes, actual 115034 bytes) context:   line 1916: "G1 X328.968 Y328.968"   line 1917: expected "M73 P41 R2"; actual "G1 X329.64 Y329.64"   line 1918: expected "G1 X329.64 Y329.64"; actual "M73 P41 R2" |
| PASS | Volumic/EXO65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.4 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.6 nozzle) |  |
| PASS | Volumic/EXO65 Performance (0.8 nozzle) |  |
| PASS | Volumic/EXO65 Stage 2 (0.6 nozzle) |  |
| DIVERGENT | Volumic/SH65 (0.4 nozzle) | first difference at byte 4836 (line 220, column 1; expected 120290 bytes, actual 120290 bytes) context:   line 219: "G1 X328.474 Y148.946 E.01769"   line 220: expected "M73 P4 R4"; actual "G1 X323.946 Y153.474 E.17881"   line 221: expected "G1 X323.946 Y153.474 E.17881"; actual "M73 P4 R4" |
| PASS | Volumic/SH65 IDRE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE COPY MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 IDRE MIRROR MODE (0.4 nozzle) |  |
| PASS | Volumic/SH65 Performance (0.4 nozzle) |  |
| PASS | Volumic/SH65 Stage 2 (0.4 nozzle) |  |
| PASS | Volumic/VS20MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK2 (0.4 nozzle) |  |
| PASS | Volumic/VS30MK3 (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30MK3 Stage 2 (0.4 nozzle) | first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30SC (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 (0.4 nozzle) |  |
| PASS | Volumic/VS30SC2 Performance (0.4 nozzle) |  |
| DIVERGENT | Volumic/VS30SC2 Stage 2 (0.4 nozzle) | first difference at byte 18421 (line 801, column 1; expected 101231 bytes, actual 101231 bytes) context:   line 800: "G1 X146.507 Y98.548 E.10493"   line 801: expected "G1 X146.507 Y96.507 E.0742"; actual "M73 P18 R3"   line 802: expected "M73 P18 R3"; actual "G1 X146.507 Y96.507 E.0742" |
| PASS | Volumic/VS30ULTRA (0.4 nozzle) |  |
| PASS | Voron/Voron 0.1 0.15 nozzle |  |
| DIVERGENT | Voron/Voron 0.1 0.2 nozzle | first difference at byte 333887 (line 12739, column 62; expected 354370 bytes, actual 354370 bytes) context:   line 12738: "; estimated printing time (normal mode) = 8m 24s"   line 12739: expected "; estimated first layer printing time (normal mode) = 0.486746s"; actual "; estimated first layer printing time (normal mode) = 0.486747s"   line 12740: "" |
| PASS | Voron/Voron 0.1 0.25 nozzle |  |
| PASS | Voron/Voron 0.1 0.4 nozzle |  |
| PASS | Voron/Voron 0.1 0.5 nozzle |  |
| PASS | Voron/Voron 0.1 0.6 nozzle |  |
| PASS | Voron/Voron 0.1 0.8 nozzle |  |
| PASS | Voron/Voron 0.1 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 250 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 250 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 300 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 300 1.0 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.15 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.2 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.25 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.4 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.5 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.6 nozzle |  |
| PASS | Voron/Voron 2.4 350 0.8 nozzle |  |
| PASS | Voron/Voron 2.4 350 1.0 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.15 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.2 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.25 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.4 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.5 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.6 nozzle |  |
| PASS | Voron/Voron Switchwire 250 0.8 nozzle |  |
| PASS | Voron/Voron Switchwire 250 1.0 nozzle |  |
| PASS | Voron/Voron Trident 250 0.15 nozzle |  |
| PASS | Voron/Voron Trident 250 0.2 nozzle |  |
| PASS | Voron/Voron Trident 250 0.25 nozzle |  |
| PASS | Voron/Voron Trident 250 0.4 nozzle |  |
| PASS | Voron/Voron Trident 250 0.5 nozzle |  |
| PASS | Voron/Voron Trident 250 0.6 nozzle |  |
| PASS | Voron/Voron Trident 250 0.8 nozzle |  |
| PASS | Voron/Voron Trident 250 1.0 nozzle |  |
| PASS | Voron/Voron Trident 300 0.15 nozzle |  |
| PASS | Voron/Voron Trident 300 0.2 nozzle |  |
| PASS | Voron/Voron Trident 300 0.25 nozzle |  |
| PASS | Voron/Voron Trident 300 0.4 nozzle |  |
| PASS | Voron/Voron Trident 300 0.5 nozzle |  |
| PASS | Voron/Voron Trident 300 0.6 nozzle |  |
| PASS | Voron/Voron Trident 300 0.8 nozzle |  |
| PASS | Voron/Voron Trident 300 1.0 nozzle |  |
| PASS | Voron/Voron Trident 350 0.15 nozzle |  |
| PASS | Voron/Voron Trident 350 0.2 nozzle |  |
| PASS | Voron/Voron Trident 350 0.25 nozzle |  |
| PASS | Voron/Voron Trident 350 0.4 nozzle |  |
| PASS | Voron/Voron Trident 350 0.5 nozzle |  |
| PASS | Voron/Voron Trident 350 0.6 nozzle |  |
| PASS | Voron/Voron Trident 350 0.8 nozzle |  |
| PASS | Voron/Voron Trident 350 1.0 nozzle |  |
| PASS | Voxelab/Voxelab Aquila X2 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 235 AWD 0.6 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.4 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.5 nozzle |  |
| PASS | Vzbot/Vzbot 330 AWD 0.6 nozzle |  |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.2mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.3mm nozzle | first difference at byte 2070 (line 97, column 1; expected 246953 bytes, actual 247092 bytes) context:   line 96: "G1 X156.463 Y142.474 E.03406"   line 97: expected "G1 X157.532 Y143.547 E.03406"; actual "M73 P27 R13"   line 98: expected "G1 X157.92 Y145 E.0338"; actual "G1 X157.532 Y143.547 E.03406" |
| PASS | WEMAKE3D/WEMAKE3D PhoenixProV1 0.4mm nozzle |  |
| DIVERGENT | WEMAKE3D/WEMAKE3D PhoenixProV1 0.6mm nozzle | first difference at byte 1690 (line 83, column 1; expected 109754 bytes, actual 109765 bytes) context:   line 82: "G1 X157.576 Y156.477 E.10062"   line 83: expected "M73 P44 R6"; actual "G1 X156.501 Y157.562 E.10062"   line 84: expected "G1 X156.501 Y157.562 E.10062"; actual "M73 P44 R6" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.2mm nozzle | first difference at byte 3712 (line 165, column 1; expected 139401 bytes, actual 139535 bytes) context:   line 164: "G1 X101.566 Y101.566 E.16373"   line 165: expected "M73 P43 R6"; actual "G1 X108.434 Y101.566 E.16373"   line 166: expected "G1 X108.434 Y101.566 E.16373"; actual "M73 P43 R6" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.3mm nozzle | first difference at byte 1505 (line 75, column 1; expected 242589 bytes, actual 242717 bytes) context:   line 74: "G1 X100 Y96.713 E.04238"   line 75: expected "M73 P27 R13"; actual "G1 X110 Y96.713 E.22479"   line 76: expected "G1 X110 Y96.713 E.22479"; actual "M73 P27 R13" |
| DIVERGENT | WEMAKE3D/WEMAKE3D TinyBotV1 0.4mm nozzle | first difference at byte 2009 (line 96, column 1; expected 132655 bytes, actual 132654 bytes) context:   line 95: "G1 X110 Y97.784 E.56236"   line 96: expected "M73 P43 R6"; actual "G1 X111.257 Y98.175 E.07402"   line 97: expected "G1 X111.257 Y98.175 E.07402"; actual "M73 P43 R6" |
| PASS | WEMAKE3D/WEMAKE3D TinyBotV1 0.6mm nozzle |  |
| PASS | Wanhao/Wanhao D12-300 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO M2 MONO DUAL 0.4 nozzle PoopTool |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 230 PRO SMARTPAD MONO DUAL 0.4 nozzle PoopTool |  |
| PASS | Wanhao France/D12 300 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 300 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO M2 MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD DIRECT 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL 0.4 nozzle |  |
| PASS | Wanhao France/D12 500 PRO SMARTPAD MONO DUAL PoopTool 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.2 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.4 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR 0.8 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-MTU1oo") |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.4 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra 0.6 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-qQ98KP") |
| PASS | WonderMaker/WonderMaker ZR Ultra 0.8 nozzle |  |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.2 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-OEJ1wl") |
| ORCA_ERROR | WonderMaker/WonderMaker ZR Ultra S 0.4 nozzle | Slice/Process: unsuccessful Orca process; evidence Some("/tmp/ares-parity-artifacts/orca-runner-MD30yi/command-uYrp3W") |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.6 nozzle |  |
| PASS | WonderMaker/WonderMaker ZR Ultra S 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S1000 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S300 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S400 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S600 Dual 0.8 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.4 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.6 nozzle |  |
| PASS | Z-Bolt/Z-Bolt S800 Dual 0.8 nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.25 Nozzle | iQ/iQ TiQ2 0.25 Nozzle process: no compatible preset |
| PASS | iQ/iQ TiQ2 0.4 Nozzle |  |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.6 Nozzle | iQ/iQ TiQ2 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ2 0.8 Nozzle | iQ/iQ TiQ2 0.8 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.25 Nozzle | iQ/iQ TiQ8 0.25 Nozzle process: no compatible preset |
| DIVERGENT | iQ/iQ TiQ8 0.4 Nozzle | first difference at byte 22135 (line 885, column 6; expected 130340 bytes, actual 130334 bytes) context:   line 884: "G1 Z1 F18000"   line 885: expected "G1 X246.061 Y187.924 Z1"; actual "G1 X237.93 Y195.977 Z1"   line 886: "G1 Z.6" |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.6 Nozzle | iQ/iQ TiQ8 0.6 Nozzle process: no compatible preset |
| VENDOR_INCOMPLETE | iQ/iQ TiQ8 0.8 Nozzle | iQ/iQ TiQ8 0.8 Nozzle process: no compatible preset |
| DIVERGENT | re3D/re3D Gigabot 4 0.4 nozzle | first difference at byte 1375 (line 59, column 23; expected 86538 bytes, actual 86531 bytes) context:   line 58: "SET_VELOCITY_LIMIT ACCEL=1000"   line 59: expected "G1 X283.942 Y288.785 F18000"; actual "G1 X283.942 Y288.785 F6000"   line 60: expected "G1 Z.4 F6000"; actual "G1 Z.4" |
| DIVERGENT | re3D/re3D Gigabot 4 0.8 nozzle | first difference at byte 1374 (line 59, column 23; expected 61613 bytes, actual 61607 bytes) context:   line 58: "SET_VELOCITY_LIMIT ACCEL=1000"   line 59: expected "G1 X283.618 Y288.329 F18000"; actual "G1 X283.618 Y288.329 F6000"   line 60: expected "G1 Z.6 F6000"; actual "G1 Z.6" |
| DIVERGENT | re3D/re3D Gigabot 4 XLT 0.4 nozzle | first difference at byte 1375 (line 59, column 23; expected 86535 bytes, actual 86528 bytes) context:   line 58: "SET_VELOCITY_LIMIT ACCEL=1000"   line 59: expected "G1 X283.942 Y368.785 F18000"; actual "G1 X283.942 Y368.785 F6000"   line 60: expected "G1 Z.4 F6000"; actual "G1 Z.4" |
| DIVERGENT | re3D/re3D Gigabot 4 XLT 0.8 nozzle | first difference at byte 1374 (line 59, column 23; expected 61621 bytes, actual 61593 bytes) context:   line 58: "SET_VELOCITY_LIMIT ACCEL=1000"   line 59: expected "G1 X283.618 Y368.329 F18000"; actual "G1 X283.618 Y368.329 F6000"   line 60: expected "G1 Z.6 F6000"; actual "G1 Z.6" |
| DIVERGENT | re3D/re3D GigabotX 2 0.8 nozzle | first difference at byte 942 (line 52, column 1; expected 42115 bytes, actual 42799 bytes) context:   line 51: "M73 P6 R1"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "SET_VELOCITY_LIMIT ACCEL=500" |
| DIVERGENT | re3D/re3D GigabotX 2 1.75 nozzle | first difference at byte 934 (line 52, column 1; expected 31516 bytes, actual 31792 bytes) context:   line 51: "M73 P10 R0"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "M73 P11 R0" |
| DIVERGENT | re3D/re3D GigabotX 2 XLT 0.8 nozzle | first difference at byte 942 (line 52, column 1; expected 42348 bytes, actual 43032 bytes) context:   line 51: "M73 P6 R1"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "SET_VELOCITY_LIMIT ACCEL=500" |
| DIVERGENT | re3D/re3D GigabotX 2 XLT 1.75 nozzle | first difference at byte 934 (line 52, column 1; expected 31471 bytes, actual 31747 bytes) context:   line 51: "M73 P11 R0"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "M73 P12 R0" |
| DIVERGENT | re3D/re3D Terabot 4 0.4 nozzle | first difference at byte 1399 (line 59, column 23; expected 86562 bytes, actual 86566 bytes) context:   line 58: "SET_VELOCITY_LIMIT ACCEL=1000"   line 59: expected "G1 X446.442 Y446.285 F18000"; actual "G1 X446.442 Y446.285 F6000"   line 60: expected "G1 Z.4 F6000"; actual "G1 Z.4" |
| DIVERGENT | re3D/re3D Terabot 4 0.8 nozzle | first difference at byte 660 (line 20, column 9; expected 61109 bytes, actual 61092 bytes) context:   line 19: "EXCLUDE_OBJECT_DEFINE NAME=cube10.stl_id_0_copy_0 CENTER=457.5,457.5 POLYGON=[[452.5,452.5],[462.5,452.5],[462.5,462.5],[452.5,462.5],[452.5,452.5]]"   line 20: expected "M73 P0 R2"; actual "M73 P0 R3"   line 21: ";TYPE:Custom" |
| DIVERGENT | re3D/re3D TerabotX 2 0.8 nozzle | first difference at byte 942 (line 52, column 1; expected 42287 bytes, actual 42971 bytes) context:   line 51: "M73 P8 R1"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "SET_VELOCITY_LIMIT ACCEL=500" |
| DIVERGENT | re3D/re3D TerabotX 2 1.75 nozzle | first difference at byte 934 (line 52, column 1; expected 31364 bytes, actual 31640 bytes) context:   line 51: "M73 P14 R0"   line 52: expected "SET_VELOCITY_LIMIT ACCEL=500"; actual "G1 E0 F1800"   line 53: expected ";TYPE:Skirt"; actual "SET_VELOCITY_LIMIT ACCEL=500" |
