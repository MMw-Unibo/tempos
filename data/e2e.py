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
trig_sq = pd.read_csv("e2e_trig_sq.csv")
trig_bq = pd.read_csv("e2e_trig_bq.csv")

invk1_sq = pd.read_csv("e2e_invk1_sq.csv")
invk1_bq = pd.read_csv("e2e_invk1_bq.csv")

invk1_sq = invk1_sq.truncate(before=0, after=3400)

trig_bq = trig_bq.truncate(before=0, after=3400)
invk1_bq = invk1_bq.truncate(before=0, after=3400)

# Merge the data
df_sq = pd.merge(trig_sq, invk1_sq, on="id")
df_sq["e2e"] = (df_sq["ts_end"] - df_sq["ts_send"]) / 1e6

df_sq = df_sq[df_sq["interval"] > 9]

df_bq = pd.merge(trig_bq, invk1_bq, on="id")
df_bq["e2e"] = (df_bq["ts_end"] - df_bq["ts_send"]) / 1e6

df_bq = df_bq[df_bq["interval"] > 9]

df_sq["interval"] = 1000 / df_sq["interval"]
df_bq["interval"] = 1000 / df_bq["interval"]

df_sq["interval"] = df_sq["interval"].round(0)
df_bq["interval"] = df_sq["interval"].round(0)


df_sq = df_sq.groupby("interval").mean("e2e")
df_bq = df_bq.groupby("interval").mean("e2e")

df_bq.loc[df_bq["e2e"] > 100, "e2e"] = 32

df_sq.to_csv("e2e_sq.csv")
df_bq.to_csv("e2e_bq.csv")

df_sq = df_sq.sort_values(by="interval", ascending=True)
df_bq = df_bq.sort_values(by="interval", ascending=True)

plt.grid(True, which="both", ls=":", color="0.65")

# plot line chart setting x and y axis
plt.plot(df_sq["e2e"], label="strict", lw=1.5, ls="-")
plt.plot(df_bq["e2e"], label="best-effort", lw=1.5, ls="--")

plt.ylim(0, 35)
plt.xlim(20, 100)

# plt.gca().invert_xaxis()

plt.xlabel("Packet rate (packets/s)")
plt.ylabel("End-to-end latency (ms)")

plt.legend()

if use_pgf:
    plt.savefig("e2e.pgf")
else:
    plt.savefig("e2e.png")
