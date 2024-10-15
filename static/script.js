const timer_div = document.getElementById('timer');

let last_live = undefined;

fetch("/api")
  .then((response) => response.json())
  .then(json => {
    if (json["NotLive"])  {
      last_live = Date.parse(json.NotLive.last_live);
    } else if (json["Live"]) {
      timer_div.innerHTML = "";
      let inner = document.createElement('div');
      inner.id = "ytplayer";
      timer_div.appendChild(inner);

      let tag = document.createElement('script');
      tag.src = "https://www.youtube.com/player_api";
      let firstScriptTag = document.getElementsByTagName('script')[0];
      firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);

      new YT.Player('ytplayer', {
          height: '360',
          width: '640',
          videoId: json.Live.video_id,
      });
    } else {
      timer_div.innerHTML = "something went wrong when trying to get data! oopsie!!";
    }

    update();
  })

function update() {
    if (last_live) {
        const now = Date.now();
        const diff = now - last_live;
        timer_div.innerHTML = `${humanizeDuration(diff, {language: "en", round: true})} without mono...`;
    }
}

setInterval(update, 1000);

