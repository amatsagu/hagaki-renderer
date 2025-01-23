import base64
from json import dumps
from os import listdir
from random import choice, randint


def main():
    dir_content = [int(x[:-4]) for x in listdir("../cdn/public/character")]
    card_count = 5
    data = []

    for _ in range(card_count):
        data.append({
            "id": choice(dir_content),
            "frame_type": randint(0, 2),
            "glow": randint(0, 1) == 1,
            "dye": randint(0, 2**24-1),
            "target_card": False
        })

    print(base64.b64encode(dumps({"cards": data}).encode()).decode().replace('=', ''))


if __name__ == '__main__':
    main()