require 'pathname'
require 'json'
require 'chunky_png'

# Function to normalize a value (-1 to 1) to grayscale (0 to 255)
def normalize_to_grayscale(value)
  ((value + 1) * 127.5).to_i
end

def render_matrix_image matrix, path, square_size, image_file, highlight_path: false
  size_y = matrix.size * square_size
  size_x = matrix.first.size * square_size
  png = ChunkyPNG::Image.new(size_x, size_y, ChunkyPNG::Color::WHITE)

  matrix.each_with_index do |row, y|
    row.each_with_index do |value, x|
      val = normalize_to_grayscale(value)

      color = ChunkyPNG::Color.grayscale(val)
      if highlight_path && path.include?([x, y])
        color = ChunkyPNG::Color.rgb(val, val, 0) #val / 2)
      end

      (y * square_size...(y + 1) * square_size).each do |py|
        (x * square_size...(x + 1) * square_size).each do |px|
          png[px, py] = color
        end
      end
    end
  end

  png.save image_file
end

log_at = Pathname '../log'
throw "no log directory found" unless log_at.exist?

pattern = log_at + 'path*.json'
Dir[pattern.to_s].sort_by {|x| x.scan(/\d+/).map{|x| x.to_i}}.each_with_index { |path_file, i|
  output = log_at + "#{i}.png"
  output2 = log_at + "#{i}-path.png"
  next if output2.exist?

  matrix_file = path_file.sub 'path', 'matrix'
  path = JSON.parse File.read path_file
  matrix = JSON.parse File.read matrix_file

  render_matrix_image matrix, path, 1, output
  render_matrix_image matrix, path, 1, output2, highlight_path: true
}
