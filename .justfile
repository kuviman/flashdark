export-obj:
    cd {{ justfile_directory() }}
    rm -rf static/assets/level
    mkdir static/assets/level
    blender blender/roomMVP.blend --background --python blender/export.py -- static/assets/level/roomMVP.obj
