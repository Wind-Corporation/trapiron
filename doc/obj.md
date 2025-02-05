# OBJ files

OBJ files ([Wavefront OBJ file](https://en.wikipedia.org/wiki/Wavefront_.obj_file)) are a common,
very simple method of storing and exchanging 3D models. OBJ files can only include geometry data and
UV mapping; textures, materials, rigs and other common modeling components are not supported.

Trapiron engine can load OBJ files for 3D models that are not generated procedurally.

## Interpretation by Trapiron engine

OBJ files are not standardized and the data they make available can be ambiguous. This is how
Trapiron specifically treats OBJ files.

### Data processed

To successfully load an OBJ file, Trapiron requires the following data to be present in it:
- Vertex data
- Face data
- Normal data
- UV mapping

All other data is currently ignored, including per-vertex color and object headers.

### Vertices and faces

Trapiron can only process triangle faces. Quads and other non-triangle faces are not supported.

Trapiron uses Y forward, Z up, right-handed coordinate system. This matches the coordinate system
used by the rendering pipeline internally. It is also the default coordinate system of Blender,
though it does not match the default export settings for Blender; see
[Exporting from Blender](#exporting-from-blender) for more details.

Trapiron has a hard limit of 65536 vertices, thus maximum supported index is 65535. The engine
requires that the size of the index buffer is at most 6\*65536 = 393216, meaning that a maximum of
131072 faces are supported.

### Normals

Normals are interpreted in the same way as vertex positions.

### UV mapping

Only the first and second UV coordinates are used, the third UV coordinate is currently ignored.
There is currently no support for mapping different textures to a mesh loaded from an OBJ file.

Each texture occupies UV space from (0; 0) to (1; 1), with (0; 0) located in the bottom-left corner.

## Exporting from Blender

Blender is the recommended tool for creating 3D models for Trapiron. Trapiron uses the default
coordinate system of Blender (Y forward, Z up, right-handed) internally and when loading OBJ files.

However, the default Wavefront OBJ settings of Blender are not suitable for Trapiron. To export a
textured model from Blender for use in Trapiron, do the following:
1. Go to File -> Export -> Wavefront (.obj).
2. In the right-hand panel of the opened dialog, change Forward Axis to Y.
3. Notice Up Axis change to Z automatically.
4. Check Geometry -> Triangulated Mesh. Trapiron does not support quads and other non-triangle
faces.
5. Uncheck Materials -> Export checkbox. Materials are not supported by Trapiron; textures are
loaded separately.
6. Select export path and click Export Wavefront OBJ to complete.

To avoid configuring these settings manually every time, save them as an operator preset using the
top of the right-hand panel of the export dialog before completing the export.

Use similar settings to import Trapiron OBJ files if necessary.
