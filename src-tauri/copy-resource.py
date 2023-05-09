import os
import shutil

curdir = os.path.abspath(os.curdir)
if not curdir.endswith('/src-tauri'):
  curdir+='/src-tauri'

if os.path.exists(f"{curdir}/model"):
  shutil.rmtree(f"{curdir}/model")
shutil.copytree(f'{curdir}/cpp/model', curdir+"/model")
library = [
  'opencv_core4',
  'opencv_imgcodecs4',
  'opencv_imgproc4',
  'opencv_videoio4',
  'avformat-59',
  'swscale-6',
  'avcodec-59',
  'jpeg62',
  'avutil-57',
  'swresample-4',
  'onnxruntime'
]
for lib in os.listdir(f'{curdir}/cpp/build'):
  if not lib.endswith('.dll'):
    continue
  basename = os.path.basename(lib)
  if '.' in basename:
    basename = basename[0:basename.index('.')]
  else:
    continue
  if basename not in library:
    print(f"Ignore {basename}")
    continue
  if not os.path.exists(f"{curdir}/{lib}"):
    print(f"copy {lib}")
    shutil.copy(f"{curdir}/cpp/build/{lib}", f"{curdir}/{lib}")