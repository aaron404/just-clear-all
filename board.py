import os
import subprocess as sp
import sys
import time

import random

import numpy as np
from PIL import Image

BOARD_SIZE = 8

colors = {
    (12, 14, 6): 1, 
    (11, 14, 5): 1, 
    (6, 11, 14): 2, 
    (5, 11, 15): 2, 
    (15, 12, 6): 3,
    (15, 12, 5): 3, 
    (15, 6, 4): 4,
    (0, 10, 9): 5,
    (2, 10, 9): 5,
    (8, 6, 14): 6,
    (7, 6, 15): 6,
    (15, 5, 11): 7,
    (15, 5, 12): 7,
    (15, 6, 11): 7,
    (14, 2, 4): 8,
    (14, 4, 4): 8,
    (14, 3, 4): 8,
    (15, 15, 3): 9,
    (11, 8, 8): 10,
    (11, 8, 7): 10,
    (8, 0, 4): 11
}

def get_avg_col(image):
    w, h = image.size
    data = list(image.getdata())
    total = [0, 0, 0]
    for d in data:
        total[0] += d[0]
        total[1] += d[1]
        total[2] += d[2]
    total = tuple(int(t / (w * h)) // 16 for t in total)
    return total

def parse_board(image):
    offset = (31, 850)
    tilesize = 127
    margin1 = 2
    margin2 = 96
    board = np.zeros((BOARD_SIZE, BOARD_SIZE), dtype=np.uint8)# [[0 for i in range(BOARD_SIZE)] for j in range(BOARD_SIZE)]
    for y in range(BOARD_SIZE):
        for x in range(BOARD_SIZE):
            x1 = offset[0] + x * tilesize + margin1
            y1 = offset[1] + y * tilesize + margin1
            x2 = offset[0] + (x+1) * tilesize - margin2
            y2 = offset[1] + (y+1) * tilesize - margin2
            tile = image.crop((x1, y1, x2, y2))
            #tile = tile.reduce(2)
            avg_col = get_avg_col(tile)
            if not avg_col in colors:
                num = int(input(f"number for <{x},{y}> {avg_col}: "))
                colors[avg_col] = num
            else:
                pass
                #print(avg_col, colors[avg_col])
            if random.random() < 0.0:
                board[y][x] = 0
            else:
                board[y][x] = colors[avg_col]
    return board

def tap(x, y, delay=0.75):
    os.system(f"adb shell input tap {x} {y}")
    time.sleep(delay)

def solve(moves):
    delta = 127
    offsetX =  31 + delta // 2
    offsetY = 850 + delta // 2

    for x, y in moves:
        tx = x * delta + offsetX
        ty = y * delta + offsetY
        tap(tx, ty)
        tap(tx, ty)

if __name__ == "__main__":

    # os.system("adb exec-out screencap -p > screen.png")

    img = Image.open("screen_fast.png")
    board = parse_board(img)
    board = " ".join(" ".join(str(c) for c in row) for row in board)

    # import random
    # board = " ".join([str(random.randint(0, 2)) for i in range(64)])

    print(board)
    
    proc = sp.Popen(f"target\\release\\jca.exe {sys.argv[1]} {board}", stdout=sp.PIPE, universal_newlines=True)
    output = ""
    try:
        for line in iter(proc.stdout.readline, ""):
            output += line
            print(line, end="", flush=True)
    except KeyboardInterrupt:
        pass

    idx = -1
    if "time" in output.splitlines()[-1]:
        idx = -2
    moves = eval(output.splitlines()[idx].split("moves:")[1])
    print(moves)

    # solve(moves)

    # board = sys.argv[1]
    # rows = [board[i*8:(i+1)*8] for i in range(8)]

    # res = "[\n"
    # for row in rows:
    #     res += f"    [{', '.join(c for c in row)}],\n"
    # res += "]"

    # print(res)
