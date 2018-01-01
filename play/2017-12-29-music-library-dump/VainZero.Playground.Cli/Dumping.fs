namespace VainZero.MusicLibrarian

open Utf8Json
open VainZero.MusicLibrarian.JsonSerialization

type JsonFormat() =
  let formatterResolver =
    VainZero.Utf8JsonExtensions.FSharpFormatterResolver(Utf8Json.Resolvers.StandardResolver.ExcludeNullSnakeCase)

  member __.Serialize<'T>(value: 'T) =
    JsonSerializer.Serialize<'T>(value, formatterResolver)

  member this.PrettyPrint<'T>(value: 'T) =
    JsonSerializer.PrettyPrint(this.Serialize<'T>(value))

type DatabaseJsonFormat() =
  let jsonFormat = JsonFormat()

  let toMusicMetadataDto (metadata: MusicMetadata): MusicMetadataDto =
    {
      MusicMetadataDto.Location =
        metadata.FilePath
      Title =
        metadata.Title
      Performers =
        metadata.Performers
      Composers =
        metadata.Composers
      Album =
        metadata.Album
      TrackNumber =
        metadata.TrackNumber
      ReleaseYear =
        metadata.ReleaseYear
    }

  let toMusicTrackDto (track: MusicTrack): MusicTrackDto =
    {
      Location =
        track.Location
      Title =
        track.Title
      Creator =
        track.Creator
      Album =
        track.Album
    }

  let toPlaylistDto (playlist: Playlist) =
    {
      PlaylistDto.Location =
        playlist.Location
      Tracks =
        playlist.Tracks |> Array.map toMusicTrackDto
    }

  let toDatabaseDto (database: Database): DatabaseDto =
    {
      MusicMetadatas =
        database.MusicRepository.FindAll()
        |> Array.map toMusicMetadataDto
      Playlists =
        database.PlaylistRepository.FindAll()
        |> Array.map toPlaylistDto
    }

  member this.ToJson(database: Database) =
    this.DtoToJson(database |> toDatabaseDto)

  member __.DtoToJson(databaseDto: DatabaseDto) =
    jsonFormat.PrettyPrint(databaseDto)
