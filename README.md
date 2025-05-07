This is an interesting project. It uses the OSM data for Great Britain (roads, towns, labels etc.) and renders it live using Skia.

There is no optimisation, so when zoomed out it's lucky to get 1 frame per second! But zoomed in can see all the details including small private roads. I render everything, nothing is cached.

This version has bad outlines for coastal areas due to a mismatch of data. I'll likely revisit it sometime and finish that.

![Screenshot1](Screenshot1.png)

![Screenshot2](Screenshot2.png)

![Screenshot3](Screenshot3.png)
