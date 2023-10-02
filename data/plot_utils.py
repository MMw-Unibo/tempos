def set_color_map(plt, cm: str) -> bool:
    """
    Sets the colormap for the specified plot:
        - https://matplotlib.org/stable/tutorials/colors/colormaps.html
    """
    # to change default color cycle
    plt.rcParams["image.cmap"] = cm
    colormaps = plt.cm.__dict__
    if cm in colormaps:
        plt.rcParams["axes.prop_cycle"] = plt.cycler(color=colormaps[cm].colors)
        return True
    else:
        return False
