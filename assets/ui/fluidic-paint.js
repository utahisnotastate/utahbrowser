/**
 * CSS Paint worklet — fluidic geometry (Houdini-style GPU-friendly fill).
 * Registers as paint(fluidic-geometry) when supported.
 */
if (typeof registerPaint === 'function') {
  registerPaint(
    'fluidic-geometry',
    class {
      static get inputProperties() {
        return ['--fluid-color', '--fluid-density'];
      }
      paint(ctx, geom, props) {
        var w = geom.width;
        var h = geom.height;
        var color = props.get('--fluid-color').toString() || '#00f2ff';
        var density = parseFloat(props.get('--fluid-density')) || 0.5;
        var g = ctx.createLinearGradient(0, 0, w, h);
        g.addColorStop(0, color);
        g.addColorStop(0.5, 'rgba(212, 175, 55, ' + (0.15 + density * 0.3) + ')');
        g.addColorStop(1, 'transparent');
        ctx.fillStyle = g;
        ctx.beginPath();
        ctx.moveTo(0, h * 0.2);
        ctx.bezierCurveTo(w * 0.3, h * 0.05, w * 0.7, h * 0.35, w, h * 0.15);
        ctx.lineTo(w, h);
        ctx.lineTo(0, h);
        ctx.closePath();
        ctx.fill();
      }
    }
  );
}
