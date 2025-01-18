const timer_div = document.getElementById('timer');
let last_live = undefined;


let state_raw = document.body.dataset.state;
state_raw = state_raw.split(/ (.*)/s)
const state = state_raw[0];
const param = state_raw[1];

switch (state) {
    case "l":
        const reps_div = document.getElementById('reps');
        reps_div.remove();
        load_live(param);
        break;
    case "nl":
        last_live = Date.parse(param);
        update();
        break;
    default:
        timer_div.innerHTML = "something went wrong when trying to get data! oopsie!!";
        break;
}



function load_live(video_id) {
  const script = document.createElement('script');
  script.src="https://www.youtube.com/iframe_api";
  script.onload = function() {
    window.YT.ready(function() {
      timer_div.innerHTML = "";
      let inner = document.createElement('div');
      inner.id = "ytplayer";
      timer_div.appendChild(inner);

      let tag = document.createElement('script');
      tag.src = "https://www.youtube.com/iframe_api";
      let firstScriptTag = document.getElementsByTagName('script')[0];
      firstScriptTag.parentNode.insertBefore(tag, firstScriptTag);
      player_video_id = video_id;
    });
  }
  document.body.appendChild(script);
}

let player_video_id = undefined;
function onYouTubeIframeAPIReady() {
    if (player_video_id) {
        new YT.Player('ytplayer', {
            height: '360',
            width: '640',
            videoId: player_video_id,
        });
    }
}


function update() {
    if (last_live) {
        const now = Date.now();
        const diff = now - last_live;
        timer_div.innerHTML = `${humanizeDuration(diff, {language: "en", round: true})} without mono...`;
    }
}

setInterval(update, 1000);

