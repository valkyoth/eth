#!/usr/bin/env ruby
# frozen_string_literal: true

require "yaml"

SHA = /\A[0-9a-fA-F]{40}\z/

def workflow_files(directory)
  Dir.glob(File.join(directory, "*.{yml,yaml}")).sort
end

def walk(value, path, checkouts)
  case value
  when Hash
    value.each do |key, child|
      validate_uses(child, path, checkouts) if key == "uses"
      walk(child, path, checkouts)
    end
  when Array
    value.each { |child| walk(child, path, checkouts) }
  end
end

def validate_uses(value, path, checkouts)
  raise "#{path}: uses must be a string" unless value.is_a?(String)

  return if value.start_with?("./", "docker://")

  action, reference = value.rpartition("@").values_at(0, 2)
  if action.empty? || reference.empty?
    raise "#{path}: remote action has no ref: #{value}"
  end
  unless SHA.match?(reference)
    raise "#{path}: remote action is not SHA-pinned: #{value}"
  end

  checkouts << value if action == "actions/checkout"
end

directory = ARGV.fetch(0) do
  warn "usage: check_action_pins.rb WORKFLOW_DIR [--list-checkouts]"
  exit 2
end
mode = ARGV.fetch(1, "--validate")
unless ["--validate", "--list-checkouts"].include?(mode)
  warn "unknown mode: #{mode}"
  exit 2
end

checkouts = []
begin
  workflow_files(directory).each do |path|
    document = YAML.safe_load_file(
      path,
      permitted_classes: [],
      permitted_symbols: [],
      aliases: false
    )
    walk(document, path, checkouts)
  end
rescue Psych::Exception, RuntimeError => error
  warn error.message
  exit 1
end

puts checkouts if mode == "--list-checkouts"
