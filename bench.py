from argparse import ArgumentParser

from requests import get
import base64
from os import listdir
from random import choice, randint
from sys import argv
from json import dumps

parser = ArgumentParser(
    argv[0],
    description='Benchmark for hagaki service',
)
parser.add_argument(
    "-a", "--address", type=str, default="127.0.0.1:8899",
    help='Address of hagaki service'
)
parser.add_argument(
    "-s", "--service", 
    choices=["card", "fan", "album", "all"],
    default="card",
    help='Type of render'
)
parser.add_argument(
    "-n", "--number", type=int, default=5,
    help='Number of cards in each render (fan and album only)',
)
parser.add_argument(
    "-r", "--repeats", type=int, default=1000,
    help='Number of renders'
)
args = parser.parse_args()

dir_content = [int(x[:-4]) for x in listdir("../cdn/public/character")]

def get_render_time(address, service, hash):
    url = f'http://{address}/render/{service}/{hash}'
    resp = get(url)
    try:
        return float(resp.headers['x-processing-time'][:-2])
    except Exception as err:
        print(url)
        print(resp)
        print(resp.text)
        print(err)
        exit(1)

def get_random_card():
    return {
        "id": choice(dir_content),
        "target_card": False,
        "frame_type": randint(0, 2),
        "glow": randint(0, 1) == 1,
        "dye": randint(0, 2**24-1)
    }

def get_random_batch():
    batch = []
    for _ in range(args.number):
        batch.append(get_random_card())
    return {"cards": batch}

def bench_card():
    total = 0
    for _ in range(args.repeats):
        hash = base64.urlsafe_b64encode(dumps(get_random_card()).encode()).decode().replace('=', '')
        total += get_render_time(args.address, 'card', hash)
    return total

def bench_fan():
    total = 0
    for _ in range(args.repeats):
        hash = base64.urlsafe_b64encode(dumps(get_random_batch()).encode()).decode().replace('=', '')
        total += get_render_time(args.address, 'fan', hash)
    return total

def bench_album():
    total = 0
    for _ in range(args.repeats):
        hash = base64.urlsafe_b64encode(dumps(get_random_batch()).encode()).decode().replace('=', '')
        total += get_render_time(args.address, 'album', hash)
    return total

def main():

    print("Starting benchmark...")
    match args.service:
        case "card":
            print("Benchmarking card renders...")
            total = bench_card()
            print(f"Average render time: {total / args.repeats} ms")
        case "fan":
            print("Benchmarking fan renders...")
            total = bench_fan()
            print(f"Average render time: {total / args.repeats} ms")
        case "album":
            print("Benchmarking album renders...")
            total = bench_album()
            print(f"Average render time: {total / args.repeats} ms")
        case "all":
            print("Benchmarking card renders...")
            total = bench_card()
            print(f"Average render time: {total / args.repeats} ms")
            print("Benchmarking fan renders...")
            total = bench_fan()
            print(f"Average render time: {total / args.repeats} ms")
            print("Benchmarking album renders...")
            total = bench_album()
            print(f"Average render time: {total / args.repeats} ms")
        case _:
            print("Invalid service")
    print("Done")

if __name__ == '__main__':
    main()