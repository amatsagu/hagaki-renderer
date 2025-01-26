from requests import get
import base64
from os import listdir
from random import choice, randint


def get_render_time(hash):
    url = f'https://hagaki.kikuri-bot.xyz/render/card/{hash}'
    return float(get(url).headers['X-Processing-Time'])


def main():
    dir_content = [x[:-4] for x in listdir("../cdn/public/character")]
    # print("preparing data...")
    # for char in dir_content:
    #     data = f'{{"id":{char},"target_card":false,"frame_type":{randint(0, 2)},"glow":{str(randint(0, 1) == 1).lower()},"dye":{randint(0, 2**24-1)}}}'
    #     hash = base64.b64encode(data.encode()).decode().replace('=', '')
    #     get_render_time(hash)
    print("Starting benchmark...")
    average = 0
    for _ in range(1000):
        char = choice(dir_content)
        data = f'{{"id":{char},"target_card":false,"frame_type":{randint(0, 2)},"glow":{str(randint(0, 1) == 1).lower()},"dye":{randint(0, 2**24-1)}}}'
        hash = base64.b64encode(data.encode()).decode().replace('=', '')
        average += get_render_time(hash)
    print(average / 1000)


if __name__ == '__main__':
    main()