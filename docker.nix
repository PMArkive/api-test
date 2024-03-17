{
  dockerTools,
  demostf-api-test,
}:
dockerTools.buildLayeredImage {
  name = "demostf/api-test";
  tag = "latest";
  maxLayers = 5;
  contents = [
    demostf-api-test
  ];
  config = {
    Cmd = ["api-test"];
  };
}