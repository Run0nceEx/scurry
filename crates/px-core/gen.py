import random


for _ in range(20000000):
    ip = '.'.join(str(random.randint(1, 255)) for _ in range(4))
    print("{}:{}".format(ip, random.randint(100, 60000)))