const imgElement = document.getElementById('image');
const nextImage = new Image();
const gallery = posts[Math.floor(Math.random() * posts.length)];
let index = 0;

window.onload = function() {
  imgElement.src = gallery[index];
  loadNextImage();
  document.body.addEventListener('click', changeImage);
};

function loadNextImage() {
  index = (index + 1) % gallery.length;
  nextImage.src = gallery[index];
}

function changeImage() {
  imgElement.src = nextImage.src;
  loadNextImage();
}
