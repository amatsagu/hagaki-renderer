from requests import get
import base64
from os import listdir
from random import choice


def get_render_time(hash):
    url = f'http://127.0.0.1:3000/render/card/{hash}'
    return float(get(url).headers['X-Processing-Time'])


def main():
    dir_content = [x[:-4] for x in listdir("../cdn/public/character")]
    average = 0
    for i in range(1000):
        char = choice(dir_content)
        hash = base64.b64encode(f'{{"id":{char},"frame":2,"glow":true,"dye":2616580}}'.encode()).decode().replace('=', '')
        # print(hash)
        # return
        average += get_render_time(hash)
    print(average / 1000)


if __name__ == '__main__':
    main()