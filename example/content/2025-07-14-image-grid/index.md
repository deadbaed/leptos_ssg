+++
title = "Create a grid of images"
date = 2025-07-14T01:46:37+02:00[Europe/Paris]
uuid = "093165b4-796c-44c5-9e01-3792762bf67f"
+++

A feature of `leptos_ssg` is the ability to take a directory containing images, and rendering them in a grid on the HTML page.

The grid will not be rendered for the web feed. I recommend to provide a link to the directory directly in a paragraph next to the grid, and let your web server present the list of files.

## In the markdown

Insert this custom HTML tag (which does not exist):

```html
&lt;ImageGrid src="path-to-images/" /&gt;
```

This will tell `leptos_ssg` to render a leptos component with a grid showing images with links to their file.

## In action

See it below:

<ImageGrid src="logos/" />
