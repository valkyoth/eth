#!/usr/bin/env ruby
# frozen_string_literal: true

require "yaml"

SHA = /\A[0-9a-fA-F]{40}\z/
SHA256_IMAGE = /\A[^\s@]+@sha256:[0-9a-fA-F]{64}\z/

def workflow_files(directory)
  Dir.children(directory)
    .select { |name| [".yml", ".yaml"].include?(File.extname(name)) }
    .map { |name| File.join(directory, name) }
    .select { |path| File.file?(path) }
    .sort
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

  return if value.start_with?("./")

  if value.start_with?("docker://")
    validate_image(value.delete_prefix("docker://"), path, "Docker action")
    return
  end

  action, reference = value.rpartition("@").values_at(0, 2)
  if action.empty? || reference.empty?
    raise "#{path}: remote action has no ref: #{value}"
  end
  unless SHA.match?(reference)
    raise "#{path}: remote action is not SHA-pinned: #{value}"
  end

  if action.casecmp?("actions/checkout")
    unless action == "actions/checkout"
      raise "#{path}: actions/checkout must use canonical lowercase spelling"
    end
    checkouts << value
  end
end

def validate_image(value, path, kind)
  raise "#{path}: #{kind} image must be a string" unless value.is_a?(String)
  unless SHA256_IMAGE.match?(value)
    raise "#{path}: #{kind} must be pinned to a SHA-256 digest: #{value}"
  end
end

def validate_job_images(document, path)
  return unless document.is_a?(Hash) && document["jobs"].is_a?(Hash)

  document["jobs"].each_value do |job|
    next unless job.is_a?(Hash)

    container = job["container"]
    if container.is_a?(Hash)
      validate_image(container["image"], path, "job container")
    elsif !container.nil?
      validate_image(container, path, "job container")
    end

    services = job["services"]
    next if services.nil?
    raise "#{path}: services must be a mapping" unless services.is_a?(Hash)

    services.each_value do |service|
      raise "#{path}: service must be a mapping" unless service.is_a?(Hash)
      validate_image(service["image"], path, "service container")
    end
  end
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
    validate_job_images(document, path)
  end
rescue Psych::Exception, RuntimeError => error
  warn error.message
  exit 1
end

puts checkouts if mode == "--list-checkouts"
