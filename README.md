# Meme Management

<div align="center">
<table>
<tr>
<td>
<img width="128" src="src-tauri/icons/128x128.png">
</td>
</tr>
</table>
</div>

![GitHub](https://img.shields.io/github/license/mslxl/meme-management?style=for-the-badge)
![Milestones](https://img.shields.io/github/milestones/all/mslxl/meme-management?style=for-the-badge)
![Release](https://img.shields.io/github/v/release/mslxl/meme-management?include_prereleases&style=for-the-badge)


Meme Management(abbr. `MMM`) is wrote for manage and locate images(most of these is meme image), but it can store other image too. MMM use [tauri](https://github.com/tauri-apps/tauri) as its user interface and SQLite as database, so the execuate file size is small.


MMM use tags like EHentai to find corresponding meme image, each tag has its namespace and value, e.g. the namespace of tag `character:yukikaze` is `character`, and its value is `yukikaze`. So it is fast to locate image and send it to social media. In case user to adjust the text in image, MMM provided a basic image editor, user can adjust image to match dialogue context.
