# Amboise-Wallpaper

This project enables you setting a real-time wallpaper based on images provided by a camera located in [Amboise](https://fr.wikipedia.org/wiki/Amboise).
The camera is filming the [Pagode de Chanteloup](https://fr.wikipedia.org/wiki/Ch%C3%A2teau_de_Chanteloup_(Indre-et-Loire)).

## Backend

It includes a simple backend that crawls the latest image and updates it on disk. It involves retry mecanisms and timestamps.
This offsets complexity for the actual wallpaper-updating apps.
I am hosting [one public instance](https://amboise.dera.page).

## GNOME

```
cargo install --git https://github.com/Mubelotix/realtime-background realtime-background && realtime-background
```

## Browsers

Install [the TablissNG extension](https://github.com/BookCatKid/TablissNG). Press the gear icon in the corner to open the settings, choose "Online Image" in the Background category at the top, and paste `https://amboise.dera.page` as the URL.
