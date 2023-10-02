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

plt.rcParams["errorbar.capsize"] = 5


plot_utils.set_color_map(plt, "Set1")

invk1_sq = pd.read_csv("bar_invk1_sq.csv")
invk2_sq = pd.read_csv("bar_invk2_sq.csv")

invk1_sq_comp = invk1_sq[invk1_sq["func"] == "comp"]
invk1_sq_encrypt = invk1_sq[invk1_sq["func"] == "encrypt"]
invk2_sq_decrypt = invk2_sq[invk2_sq["func"] == "decrypt"]
invk2_sq_decomp = invk2_sq[invk2_sq["func"] == "decomp"]

diff_sq_comp = invk1_sq_comp["ts_end"] - invk1_sq_comp["ts_start"]
diff_sq_enc = invk1_sq_encrypt["ts_end"] - invk1_sq_encrypt["ts_start"]
diff_sq_dec = invk2_sq_decrypt["ts_end"] - invk2_sq_decrypt["ts_start"]
diff_sq_dcp = invk2_sq_decomp["ts_end"] - invk2_sq_decomp["ts_start"]

diff_sq_comp = diff_sq_comp / 1e6
diff_sq_enc = diff_sq_enc / 1e6
diff_sq_dec = diff_sq_dec / 1e6
diff_sq_dcp = diff_sq_dcp / 1e6

invk1_bq = pd.read_csv("bar_invk1_bq.csv")
invk2_bq = pd.read_csv("bar_invk2_bq.csv")

invk1_bq_comp = invk1_bq[invk1_bq["func"] == "comp"]
invk1_bq_encrypt = invk1_bq[invk1_bq["func"] == "encrypt"]
invk2_bq_decrypt = invk2_bq[invk2_bq["func"] == "decrypt"]
invk2_bq_decomp = invk2_bq[invk2_bq["func"] == "decomp"]

diff_bq_comp = invk1_bq_comp["ts_end"] - invk1_bq_comp["ts_start"]
diff_bq_enc = invk1_bq_encrypt["ts_end"] - invk1_bq_encrypt["ts_start"]
diff_bq_dec = invk2_bq_decrypt["ts_end"] - invk2_bq_decrypt["ts_start"]
diff_bq_dcp = invk2_bq_decomp["ts_end"] - invk2_bq_decomp["ts_start"]

diff_bq_comp = diff_bq_comp / 1e6
diff_bq_enc = diff_bq_enc / 1e6
diff_bq_dec = diff_bq_dec / 1e6
diff_bq_dcp = diff_bq_dcp / 1e6

# save diff_bq_comp to csv
diff_bq_comp.to_csv("diff_bq_comp.csv", index=False)

sq_means = [
    diff_sq_comp.mean(),
    diff_sq_enc.mean(),
    diff_sq_dec.mean(),
    diff_sq_dcp.mean(),
]
bq_means = [
    diff_bq_comp.mean(),
    diff_bq_enc.mean(),
    diff_bq_dec.mean(),
    diff_bq_dcp.mean(),
]

sq_stds = [diff_sq_comp.std(), diff_sq_enc.std(), diff_sq_dec.std(), diff_sq_dcp.std()]
bq_stds = [diff_bq_comp.std(), diff_bq_enc.std(), diff_bq_dec.std(), diff_bq_dcp.std()]

# Plot the data using bar chart
fig, ax = plt.subplots()

ax.grid(linestyle=":", which="both", axis="both")

x = np.arange(4)
width = 0.35

rects1 = ax.bar(x - width / 2, sq_means, width, yerr=sq_stds, label="Strict Quality")
rects2 = ax.bar(
    x + width / 2, bq_means, width, yerr=bq_stds, label="Best-effort Quality"
)

for rect in rects1:
    rect.set_hatch("//")

ax.set_ylabel("Time (ms)")
ax.set_xticks(x)
ax.set_xticklabels(["Compression", "Encryption", "Decryption", "Decompression"])

ax.legend()

if use_pgf:
    plt.savefig("bar.pgf")
else:
    plt.savefig("bar.png")
