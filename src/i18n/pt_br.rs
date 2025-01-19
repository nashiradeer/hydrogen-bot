//! Brazilian Portuguese translations.

use phf::{phf_map, Map};

// This macro generates a static map with the translations.
pub static TRANSLATIONS: Map<&'static str, &'static str> = phf_map! {
    "generic.embed_footer" => "Hydrogen por Nashira Deer",
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
    "play.embed_title" => "Enfileirando/Tocando músicas",
    "play.play_single" => "Tocando: **{name}** by **{author}**.",
    "play.play_single_url" => "Tocando: [**{name}**]({url}) por **{author}**.",
    "play.play_multi" => "**{count}** músicas de sua playlist foram enfileirados, **{name}** por **{author}** foi selecionada para tocar agora.",
    "play.play_multi_url" => "**{count}** músicas de sua playlist foram enfileirados, [**{name}**]({url}) por **{author}** foi selecionada para tocar agora.",
    "play.enqueue_single" => "**{name}** por **{author}** foi adicionado na fila.",
    "play.enqueue_single_url" => "[**{name}**]({url}) por **{author}** foi adicionado na fila.",
    "play.enqueue_multi" => "**{count}** músicas da sua playlist foram enfileirados.",
    "play.not_found" => "Eu não pude encontrar a música solicitada.",
    "play.truncated" => "Você não pode adicionar mais músicas na queue uma vez que ela já esteja no limite permitido. Por favor remova umas algumas músicas antes de tentar de novo.",
    "play.truncated_warn" => "**Aviso: Eu preciso ignorar algumas músicas da sua playlist porque ela maior que o limite permitido.**",
    "player.empty" => "_Atualmente não estou tocando nada._",
    "player.timeout" => "Não há mais ninguém conectado no chat de voz. Eu estarei saindo em {time} segundos.",
    "join.name" => "entrar",
    "join.description" => "Me faça entrar no chat de voz sem tocar nada.",
    "join.embed_title" => "Entrando no chat de voz",
    "join.joined" => "Eu entrei no seu chat de voz, e agora você pode pedir qualquer música usando {play}.",
    "stop.embed_title" => "Parando o tocador de música",
    "stop.stopped" => "Eu estou saindo do chat de voz. Espero te ver em breve.",
    "loop.embed_title" => "Repetindo a fila",
    "loop.looping" => "A repetição da fila foi alterado para **{loop}**.",
    "loop.autostart" => "Normal",
    "loop.no_autostart" => "Normal sem auto-reprodução",
    "loop.music" => "Repetir Música",
    "loop.queue" => "Repetir Fila",
    "loop.random" => "Próxima música aleatória",
    "pause.embed_title" => "Pausando/Resumindo o tocador de música",
    "pause.paused" => "Você pausou o tocador de música.",
    "pause.resumed" => "Você resumiu o tocador de música.",
    "skip.embed_title" => "Pulando para próxima música",
    "skip.skipping" => "Pulando para a música **{name}** por **{author}**.",
    "skip.skipping_url" => "Pulando para a música [**{name}**]({url}) por **{author}**.",
    "prev.embed_title" => "Voltando para a música anterior",
    "prev.returning" => "Voltando para a música **{name}** por **{author}**.",
    "prev.returning_url" => "Voltando para a música [**{name}**]({url}) por **{author}**.",
    "seek.name" => "procurar",
    "seek.description" => "Procura pelo tempo na música tocando atualmente.",
    "seek.time_name" => "tempo",
    "seek.time_description" => "Tempo em segundos ou sintaxe suportada.",
    "seek.embed_title" => "Procurando o tempo de música",
    "seek.invalid_syntax" => "Sintaxe de tempo inválida. Você pode usar números como segundos ou sufixa-los com `m` para minutos ou `h` para horas. Você também pode usar `00:00` ou `00:00:00` para definir as horas.",
    "seek.seeking" => "**{name}**\n{author}\n``{current}/{total}``\n{progress}",
    "seek.seeking_url" => "[**{name}**]({url})\n{author}\n``{current}/{total}``\n{progress}",
};
