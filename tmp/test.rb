x = begin
      puts 1
      puts 2
      puts 3
    rescue => e
      puts 4
      puts 5
    else
      puts 6
      puts 7
    ensure
      puts 8
      puts 9
end
