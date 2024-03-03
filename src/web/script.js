function setBg(el, _, color) {
  el.querySelector('polygon').setAttribute('fill', color);
}

function setLineColor(el, _, color) {
  el.querySelector('polygon').setAttribute('fill', color);
  el.querySelector('path').setAttribute('stroke', color);
}

function allNodes(f) {
  document.querySelectorAll('.node').forEach(el => {
    f(el, el.getAttribute('id'))
  })
}

function allEdges(f) {
  document.querySelectorAll('.edge').forEach(el => {
    f(el, el.getAttribute('id'))
  })
}

function outEdges(f, nodeId) {
  document.querySelectorAll(`.from-${nodeId}`).forEach(el => {
    f(el, el.getAttribute('id'))
  })
}

function inEdges(f, nodeId) {
  document.querySelectorAll(`.to-${nodeId}`).forEach(el => {
    f(el, el.getAttribute('id'))
  })
}

window.onload = () => {
  const urlParams = new URLSearchParams(window.location.search);
  const file = urlParams.get('file') ?? '/output/tmp.dot.svg';
  document.querySelector('.title').innerText = file;
  (
    async () => {
      const res = await fetch(file);

      console.log(document.querySelector('#svg'));
      document.querySelector('#svg').innerHTML = await res.text();


      allNodes((el, id) => {
        el.addEventListener('click', _ => {
          allNodes((el, id) => setBg(el, id, 'lightblue'))

          allEdges((el, id) => {
            setLineColor(el, id, "black")
          })
          inEdges((el, id) => {
            setLineColor(el, id, "#ff7fab")
          }, id)

          outEdges((el, edgeId) => {
            const neighborId = el.getAttribute('class').split(' ').find((item) => {
              return item.split('-')[0] === 'to'
            }).split('-')[1];
            setBg(document.querySelector(`#${neighborId}`), neighborId, '#aaaafe');


            setLineColor(el, edgeId, "#ff7fab")
          }, id)

          setBg(el, id, "#fefeaa")
        })
      })

    }
  )()

}
