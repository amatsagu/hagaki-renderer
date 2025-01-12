from requests import get


def get_render_time(hash):
    url = f'http://127.0.0.1:8899/render/card/{hash}'
    return float(get(url).headers['X-Processing-Time'][:-2])


def main():
    average = 0
    for _ in range(1000):
        average += get_render_time("eyJpZCI6MTk1NCwiZnJhbWUiOjIsImdsb3ciOnRydWUsImR5ZSI6MjYxNjU4MH0=")
    print(average / 1000)


if __name__ == '__main__':
    main()
