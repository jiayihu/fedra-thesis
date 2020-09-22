file "target/thumbv7em-none-eabi/debug/fedra"

target extended-remote :3333

# Load symbols
load

# List main file
# list main

# Jump to main function
break main

# Run up to main function
continue
