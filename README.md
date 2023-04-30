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


Meme Management(abbr. `MMM`) is wrote for managing and locating images(most of these is meme image), but it can store other image too. MMM use [tauri](https://github.com/tauri-apps/tauri) as its user interface and SQLite as database, thus the execuate file size is small.


MMM use tags like EHentai to find corresponding meme image, each tag has its namespace and value, e.g. the namespace of tag `character:yukikaze` is `character`, and its value is `yukikaze`. So it is fast to locate image and send it to social media. Also, MMM provided a basic image editor, user can adjust image to match dialogue context.

You can see the future plans in [GitHub milestone](https://github.com/mslxl/meme-management/milestones)

## Build
To build the project from sources code, you need to have the following enviroment:
- pnpm
- cargo

Currently, you can build it on your own by servel commands
```bash
$ git clone git@github.com:mslxl/meme-management.git && cd meme-management
$ pnpm install
$ pnpm run tauri build
```

## Screenshot
MMM is under developing, UI may have greate changed in following update

![](screenshots/view.png)

## Thanks

- Thanks to [MeetWq/meme-generator](https://github.com/MeetWq/meme-generator) which provide some idea for me
- Thanks to [CompVis/stable-diffusion](https://github.com/CompVis/stable-diffusion) which generate the application icon
- Thanks to my cat, though I didn't have any cat