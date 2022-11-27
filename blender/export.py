import os
import bpy
import sys

argv = sys.argv[sys.argv.index("--") + 1:]

out_path = argv[0]

bpy.ops.export_scene.obj(
    filepath=out_path, axis_forward='Y', axis_up='Z', path_mode='COPY')

out_dir = os.path.dirname(out_path)
os.chdir(out_dir)
for f in os.listdir('.'):
    if f.endswith('.png'):
        dark_f = f[:-4]+"_Dark.png"
        os.system(f"git checkout HEAD {dark_f}")
