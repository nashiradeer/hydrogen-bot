//! Brazilian Portuguese translations.

use phf::{phf_map, Map};

// This macro generates a static map with the translations.
pub static TRANSLATIONS: Map<&'static str, &'static str> = phf_map! {
    "error.unknown" => "Um erro estranho ocorreu!",
    "error.unknown_voice_state" => "Eu não pude determinar seu estado de voz, cheque minhas permissões, ou se você está em um chat de voz.",
    "error.cant_connect" => "Eu não pude entrar no seu chat de voz. Cheque se eu tenho permissões para acessar ele.",
    "error.not_in_voice_chat" => "Você não pode controlar o tocador de música de fora do chat de voz.",
    "error.player_exists" => "Já existe um tocador de música em um outro chat de voz.",
    "error.player_not_exists" => "Não tem um tocador de música nesse servidor.",
    "error.empty_queue" => "Não há músicas na fila.",
    "error.not_in_guild" => "Você não pode usar esse comando fora de um servidor.",
    "play.name" => "tocar",
    "play.description" => "Pede para uma música ser tocada, enfileirando ela na fila ou tocando imediatamente se vazio.",
    "play.query_name" => "pesquisa",
    "play.query_description" => "Uma música ou URL de uma playlist, ou um termo de pesquisa.",
    "play.play_single" => "Tocando: **{0}** by **{1}**.",
    "play.play_single_url" => "Tocando: [**{0}**](<{2}>) por **{1}**.",
    "play.play_multi" => "**{2}** músicas de sua playlist foram enfileirados, **{0}** por **{1}** foi selecionada para tocar agora.",
    "play.play_multi_url" => "**{2}** músicas de sua playlist foram enfileirados, [**{0}**](<{3}>) por **{1}** foi selecionada para tocar agora.",
    "play.enqueue_single" => "**{0}** por **{1}** foi adicionado na fila.",
    "play.enqueue_single_url" => "[**{0}**](<{2}>) por **{1}** foi adicionado na fila.",
    "play.enqueue_multi" => "**{0}** músicas da sua playlist foram enfileirados.",
    "play.not_found" => "Eu não pude encontrar a música solicitada.",
    "play.truncated" => "Você não pode adicionar mais músicas na queue uma vez que ela já esteja no limite permitido. Por favor remova umas algumas músicas antes de tentar de novo.",
    "play.truncated_warn" => "**Aviso: Eu preciso ignorar algumas músicas da sua playlist porque ela maior que o limite permitido.**",
    "player.empty" => "_Atualmente não estou tocando nada._",
    "player.timeout" => "Não há mais ninguém conectado no chat de voz. Eu estarei saindo em {0} segundos.",
    "join.name" => "entrar",
    "join.description" => "Me faça entrar no chat de voz sem tocar nada.",
    "join.joined" => "Eu entrei no seu chat de voz, e agora você pode pedir qualquer música usando {0}.",
    "stop.stopped" => "Eu estou saindo do chat de voz. Espero te ver em breve.",
    "loop.looping" => "A repetição da fila foi alterado para **{0}**.",
    "loop.autostart" => "Normal",
    "loop.no_autostart" => "Normal sem auto-reprodução",
    "loop.music" => "Repetir Música",
    "loop.queue" => "Repetir Fila",
    "loop.random" => "Próxima música aleatória",
    "pause.paused" => "Você pausou o tocador de música.",
    "pause.resumed" => "Você resumiu o tocador de música.",
    "skip.skipping" => "Pulando para a música **{0}** por **{1}**.",
    "skip.skipping_url" => "Pulando para a música [**{0}**](<{2}>) por **{1}**.",
    "prev.returning" => "Voltando para a música **{0}** por **{1}**.",
    "prev.returning_url" => "Voltando para a música [**{0}**](<{2}>) por **{1}**.",
    "seek.name" => "procurar",
    "seek.description" => "Procura pelo tempo na música tocando atualmente.",
    "seek.time_name" => "tempo",
    "seek.time_description" => "Tempo em segundos ou sintaxe suportada.",
    "seek.invalid_syntax" => "Sintaxe de tempo inválida. Você pode usar números como segundos ou sufixa-los com `m` para minutos ou `h` para horas. Você também pode usar `00:00` ou `00:00:00` para definir as horas.",
    "seek.seeking" => "**{0}**\n{1}\n``{2}/{3}``\n{4}",
    "seek.seeking_url" => "[**{0}**](<{5}>)\n{1}\n``{2}/{3}``\n{4}",
};
