import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import matplotlib as mpl

import plot_utils

use_pgf = False

mpl.use("pgf")
mpl.rcParams.update(
    {
        "pgf.texsystem": "pdflatex",
        "font.family": "serif",
        "text.usetex": True,
        "pgf.rcfonts": False,
    }
)
use_pgf = True

plot_utils.set_color_map(plt, "Set1")

# Load the data
recv1 = pd.read_csv("recv1.csv")
send1 = pd.read_csv("send1.csv")

recv2 = pd.read_csv("recv2.csv")
send2 = pd.read_csv("send2.csv")

# Merge the data
df1 = pd.merge(recv1, send1, on="id")
df1["diff"] = (df1["ts_end"] - df1["ts_start"]) / 1e3

df2 = pd.merge(recv2, send2, on="id")
df2["diff"] = (df2["ts_end"] - df2["ts_start"]) / 1e3

df1 = df1.truncate(before=500, after=1000)
df2 = df2.truncate(before=500, after=1000)

df1["id"] = df1["id"] - 500
df2["id"] = df2["id"] - 500

print(df1.head())

# plot line chart with "id" column as x-axis and "diff" column as y-axis
plt.plot(df1["id"], df1["diff"], label="strict", lw=1.5, ls="-")
plt.plot(df2["id"], df2["diff"], label="best-effort", lw=1.5, ls="--")

# plt.axvline(x=300, color="purple", lw=1.5)

#  an arrow to the point of interest (300, 1600) from text at (200, 5000)
# arrow size is 0.05
plt.annotate(
    "packets rate threshold\n(1000 packets/s)",
    xy=(300, 1600),
    xytext=(200, 10000),
    arrowprops=dict(arrowstyle="->", color="black", lw=1.5),
    color="black",
    fontsize=12,
    horizontalalignment="center",
    verticalalignment="top",
    multialignment="center",
)

plt.grid(True, which="both", ls=":", color="0.65")

plt.xlabel("Time (s)")
plt.ylabel("Latency (us)")

# set log scale
plt.yscale("log")

plt.xlim(0, 500)

plt.legend()

if use_pgf:
    plt.savefig("plot.pgf")
else:
    plt.savefig("plot.png")
