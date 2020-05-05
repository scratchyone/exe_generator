function create_exe() {
  let info = document.getElementById('info').value;
  console.log(info);
  const request = new Request('https://vps.scratchyone.com/add/' + info, {
    method: 'POST',
  });
  fetch(request)
    .then((response) => {
      if (response.status === 200) {
        return response.json();
      } else {
        throw new Error('Something went wrong on api server!');
      }
    })
    .then((response) => {
      console.log(response);
      document.getElementById('status').innerHTML =
        'Compiling... #' + response.place_in_queue + ' in queue';
      update_status(response.uuid);
      let int = window.setInterval(function () {
        update_status(response.uuid, int);
      }, 500);
    })
    .catch((error) => {
      console.error(error);
      document.getElementById('status').innerHTML = 'SERVER ERROR';
    });
}

function update_status(uuid, int) {
  const request = new Request('https://vps.scratchyone.com/info/' + uuid, {
    method: 'GET',
  });
  fetch(request)
    .then((response) => {
      if (response.status === 200) {
        return response.json();
      } else {
        throw new Error('Something went wrong on api server!');
      }
    })
    .then((response) => {
      console.log(response);
      document.getElementById('status').innerHTML = '';
      if (response.finished && response.result.success) {
        document.getElementById('status').innerHTML =
          'Finished compiling, downloading now';
        window.clearInterval(int);
        download_from_b64(response.result.output);
      } else if (response.finished) {
        document.getElementById('status').innerHTML = 'Compilation error';
        window.clearInterval(int);
      } else if (response.success) {
        document.getElementById('status').innerHTML =
          'Compiling... #' + response.place_in_queue + ' in queue';
      } else {
        document.getElementById('status').innerHTML = 'SERVER ERROR';
        window.clearInterval(int);
      }
    })
    .catch((error) => {
      console.error(error);
      document.getElementById('status').innerHTML = 'SERVER ERROR';
      window.clearInterval(int);
    });
}

function download_from_b64(pdf) {
  const linkSource = `data:application/octet-stream;base64,${pdf}`;
  const downloadLink = document.createElement('a');
  const fileName = 'custom.exe';

  downloadLink.href = linkSource;
  downloadLink.download = fileName;
  downloadLink.click();
}
