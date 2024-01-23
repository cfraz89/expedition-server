# expedition-server
The server backend for Expedition, an application to catalogue and analyse adventure motorcycle rides, uploaded from GPX files. Written in Rust, using axum web framework. Requires a postgresql database.

![screenshot](/expedition.png "screenshot")

# Features:
 - list rides, their start and end addresses, and your riding time from start and end (data from OpenStreetMap and google maps)
 - ride details, breaking down the distance travelled on each road and its surface (data from self-hosted OpenStreetMap nominatim server)
 - display ride path on a 3D map (Mapbox)
 - login using Google. Logging in doesnt give you access to anything extra currently. Auth is implemented using Ory Kratos.

# Deployment
One day