target extended-remote :3333
load
monitor reset halt
break main
c
