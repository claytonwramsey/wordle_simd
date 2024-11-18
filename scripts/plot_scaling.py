import matplotlib.pyplot as plt
import numpy as np
import sys
import pandas as pd

def main():
    # first column: L lanes
    # second column: n threads
    # third column: time taken

    df = pd.read_csv(sys.argv[1], sep=',', header=None)
    for l in df[0].unique():
        df_l = df.loc[df[0] == l, :]
        plt.plot(df_l[1], 1e9 / df_l[2], label=f"L={l}")
    plt.xlabel("Number of threads")
    plt.ylabel("Grading throughput (solns/sec)")
    plt.title("Diminishing returns from multithreading and over-laning")
    plt.legend()
    plt.savefig(sys.argv[2])


if __name__ == "__main__":
    main()
