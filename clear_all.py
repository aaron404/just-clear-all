from collections import defaultdict
import threading
import time
import sys

import  colorama
import  numpy   as np
from    PIL     import Image

import adb
import pdb

colorama.init()

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
    (7, 6, 15): 6,
    (15, 5, 11): 7,
    (15, 6, 11): 7,
    (14, 2, 4): 8
}

print_colors = defaultdict(lambda: colorama.Fore.WHITE)
print_colors.update({
    0: colorama.Fore.BLACK,
    1: colorama.Fore.GREEN,
    2: colorama.Fore.BLUE,
    3: colorama.Fore.YELLOW,
    4: colorama.Fore.MAGENTA,
})

#np.set_printoptions(formatter={'int': lambda x: f'{print_colors[x]}{x}'})

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
    offset = (30, 557)
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
            board[y][x] = colors[avg_col]
    return board

def collapse_board(board):
    # collapse vertically
    for x in range(BOARD_SIZE):
        a = board[:,x][board[:,x] > 0]
        board[:,x] = 0
        board[BOARD_SIZE-a.size:,x] = a
    for x1 in range(BOARD_SIZE):
        if not board[BOARD_SIZE-1, x1]:
            for x2 in range(x1 + 1, BOARD_SIZE):
                if board[BOARD_SIZE-1, x2]:
                    # swap columns
                    board[:, x1] = board[:, x2]
                    board[:, x2] = 0
                    break

def clone_board(board):
    return np.copy(board)

def boards_equal(board1, board2):
    return np.array_equal(board1, board2)

def get_score(k, n):
    '''Compute score of n tiles of value k'''
    multiplier = 1 + n // 5
    return k * 5 * n * multiplier

def get_bonus(board):
    global level
    n = sum(sum(board > 0))
    if n > 6:
        return 0
    return [500, 250, 200, 150, 100, 50][n - 1] * (level + 1)

def click(board, click_x, click_y):
    val = board[click_y][click_x]
    visited = np.zeros((BOARD_SIZE, BOARD_SIZE), dtype=np.bool)
    ns = [(-1, 0), (1, 0), (0, 1), (0, -1)]
    open_cells = set()
    open_cells.add((click_x, click_y))
    closed_cells = set()
    while open_cells:
        x, y = open_cells.pop()
        closed_cells.add((x, y))
        for n in ns:
            nx = x + n[0]
            ny = y + n[1]
            if nx >= 0 and nx < BOARD_SIZE and ny >= 0 and ny < BOARD_SIZE and not visited[ny][nx]:
                visited[ny][nx] = True
                if board[ny][nx] == val:
                    open_cells.add((nx, ny))
    new_board = clone_board(board)
    for x, y in closed_cells:
        new_board[y][x] = 0
        new_board[click_y][click_x] = val + 1
    collapse_board(new_board)
    score = get_score(val, len(closed_cells))
    return new_board, score

def board_clickable(board):
    can_click = np.zeros((BOARD_SIZE, BOARD_SIZE), dtype=np.bool)
    gz = board > 0
    # check left neighbors
    can_click[:,1:BOARD_SIZE-1] |= (gz[:,1:BOARD_SIZE-1]) & (board[:,0:BOARD_SIZE-2] == board[:,1:BOARD_SIZE-1])
    can_click[:,0:BOARD_SIZE-2] |= (gz[:,0:BOARD_SIZE-2]) & (board[:,1:BOARD_SIZE-1] == board[:,0:BOARD_SIZE-2])
    can_click[1:BOARD_SIZE-1,:] |= (gz[1:BOARD_SIZE-1,:]) & (board[0:BOARD_SIZE-2,:] == board[1:BOARD_SIZE-1,:])
    can_click[0:BOARD_SIZE-2,:] |= (gz[0:BOARD_SIZE-2,:]) & (board[1:BOARD_SIZE-1,:] == board[0:BOARD_SIZE-2,:])
    return can_click

def get_moves(board):
    clickable = board_clickable(board)
    all_moves = [(x, y) for x in range(BOARD_SIZE) for y in range(BOARD_SIZE) if clickable[y][x]]
    unique_moves = []
    hashes = set()
    #sub_boards = {}
    for x, y in all_moves:
        sub_board, score = click(board, x, y)
        h = hash(str(sub_board))
        if h in hashes:
            continue
        hashes.add(h)
        #sub_boards[(x, y)] = sub_board
        unique_moves.append((x, y, sub_board, score))
    return unique_moves

def solve(board, moves, score):
    global stop_threads, high_score, best_moves, perf
    if stop_threads:
        return
    valid_moves = get_moves(board)
    if not len(valid_moves):
        score += get_bonus(board)
        if score > high_score:
            perf = bool(board[BOARD_SIZE-2][0] == board[BOARD_SIZE-1][1] == 0)
            print('{: >6} {} {}'.format(score, [" ", "p"][perf], moves))
            high_score = score
            best_moves = moves
            perfect = perf
        return
    for x, y, sub_board, new_score in valid_moves:
        if len(moves) < THREAD_DEPTH:
            b = threading.Thread(target=solve, args=(sub_board, moves + [(x, y)], score + new_score), daemon=True)
            b.start()
        else:
            solve(sub_board, moves + [(x, y)], score + new_score)

class Board:
    def __init__(self, board, moves, score):
        self.board = board
        self.moves = moves
        self.scroe = score

    def run(self):
        solve(self.board, moves, score)
"""lvl 1
1000
500
400
300
200
100
lvl 2
1500
750
600
450
300
lvl 3
2000
1000
800
600

"""
THREAD_DEPTH = 1
stop_threads = False
if __name__ == "__main__":
    
    level = 1
    try:
        level = int(sys.argv[1])
    except:
        pass

    ADB = adb.ADB()

    # get screen
    ADB.getScreen()
    img = Image.open("screen.png")
    board = parse_board(img)
    print(board)

    high_score = 0
    best_moves = []
    perfect = False

    threading.Thread(target=solve, args=(board, [], 0), daemon=True).start()

    try:
        while 1:
            time.sleep(5)
    except KeyboardInterrupt:
        stop_threads = True
        

    if not input("solve? "):
        exit()

    delta = 127
    offsetX =  30 + delta // 2
    offsetY = 557 + delta // 2

    for x, y in best_moves:
        tx = x * delta + offsetX
        ty = y * delta + offsetY
        print(x, y, tx, ty)
        ADB.tap(tx, ty, 500)
        ADB.tap(tx, ty, 500)
        time.sleep(1)

    pass