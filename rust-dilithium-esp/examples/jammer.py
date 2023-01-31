from random import randint, random
from serial import Serial
from sys import argv

sender = Serial(argv[1], baudrate=115200, timeout=1e-1)
reader = Serial(argv[2], baudrate=115200, timeout=1e-1)

while True:
    line = sender.readline()
    if line:
        line_str = line.decode('ascii')
        list_str = "".join(line_str.split()[3:])[:-4]
        try:
            payload = eval(list_str)
                
            if len(payload) == 32:
                print("[jammer] forwarding a new message")
                reader.write(b"\xff\xff\xff\xff")
            else:
                print("[jammer] forwarding the signature")

            if random() < 1 / 4:
                print("[jammer] introducing an error !")
                payload[randint(0, len(payload) - 1)] = randint(0, 255)
            
            reader.write(payload)
        except Exception as e:
            pass

    line = reader.readline()
    if line:
        print(line.decode('ascii'), end="")

